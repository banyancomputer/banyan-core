use async_trait::async_trait;
use banyan_task::{CurrentTask, TaskLike};
use serde::{Deserialize, Serialize};
use sqlx::{Sqlite, Transaction};

use crate::app::AppState;
use crate::database::models::{DealState, MetadataState, SnapshotSegment};

const MAX_SNAPSHOT_SEGMENT_SIZE: i64 = 32 * 1024 * 1024 * 1024; // 32GiB
const BLOCK_SIZE: i64 = 262144; // 256 KiB
const EMPTY_DEAL: &str = ""; // 256 KiB

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum CreateDealsTaskError {
    #[error("sql error: {0}")]
    Sqlx(#[from] sqlx::Error),
}

#[derive(Deserialize, Serialize)]
pub struct CreateDealsTask {
    snapshot_id: String,
}

impl CreateDealsTask {
    pub fn new(snapshot_id: String) -> Self {
        Self { snapshot_id }
    }

    fn split_into_segments(&self, segment_size: i64) -> Vec<SnapshotSegment> {
        let mut snapshot_segments = Vec::new();
        let mut remaining_size = segment_size;
        while remaining_size > 0 {
            let segment_size = std::cmp::min(remaining_size, MAX_SNAPSHOT_SEGMENT_SIZE);
            snapshot_segments.push(SnapshotSegment {
                id: uuid::Uuid::new_v4().to_string(),
                deal_id: EMPTY_DEAL.to_string(),
                size: segment_size,
                created_at: time::OffsetDateTime::now_utc(),
                updated_at: time::OffsetDateTime::now_utc(),
            });
            remaining_size -= segment_size;
        }
        snapshot_segments
    }

    async fn create_multi_segment_deal(
        transaction: &mut Transaction<'_, Sqlite>,
        pending_snapshot_segments: &[SnapshotSegment],
        snapshot_id: &str,
    ) -> Result<(), <CreateDealsTask as TaskLike>::Error> {
        let new_deal_id = sqlx::query_scalar!(
            r#" INSERT INTO deals (state) VALUES ($1) RETURNING id;"#,
            DealState::Active,
        )
        .fetch_one(&mut **transaction)
        .await
        .map_err(CreateDealsTaskError::Sqlx)?;

        for segment in pending_snapshot_segments {
            let segment_id = sqlx::query_scalar!(
                r#"
                    INSERT INTO snapshot_segments
                    (deal_id, size)
                    VALUES ($1, $2)
                     RETURNING id;"#,
                new_deal_id,
                segment.size,
            )
            .fetch_one(&mut **transaction)
            .await
            .map_err(CreateDealsTaskError::Sqlx)?;

            let snapshot_id = snapshot_id.to_string();
            sqlx::query!(
                    r#"INSERT INTO snapshot_segment_associations (snapshot_id, segment_id) VALUES ($1, $2);"#,
                    snapshot_id,
                    segment_id,
                )
                .execute(&mut **transaction)
                .await
                .map_err(CreateDealsTaskError::Sqlx)?;
        }
        Ok(())
    }

    async fn aggregate_segments(
        transaction: &mut Transaction<'_, Sqlite>,
        pending_snapshot_segments: Vec<SnapshotSegment>,
    ) -> Result<(), <CreateDealsTask as TaskLike>::Error> {
        let single_segment_snapshots = sqlx::query_as!(
            SnapshotSegment,
            r#"SELECT ss.*
                FROM snapshot_segments ss
                 JOIN deals d ON ss.deal_id = d.id
                WHERE d.state = $1
                  AND ss.size < $2
                  AND (
                    SELECT COUNT(*)
                    FROM snapshot_segments ss2
                    WHERE ss2.deal_id = ss.deal_id
                    ) <= 1"#,
            DealState::Active,
            // do not include mulitpart segments and single segments that are 32 GiB
            MAX_SNAPSHOT_SEGMENT_SIZE
        )
        .fetch_all(&mut **transaction)
        .await
        .map_err(CreateDealsTaskError::Sqlx)?;
        let segments_for_packing = pending_snapshot_segments
            .into_iter()
            .chain(single_segment_snapshots.into_iter())
            .collect::<Vec<_>>();
        let bins = best_fit_decreasing(segments_for_packing, MAX_SNAPSHOT_SEGMENT_SIZE);

        for bin in bins {
            let new_deal_id = sqlx::query_scalar!(
                r#" INSERT INTO deals (state) VALUES ($1) RETURNING id;"#,
                DealState::Active,
            )
            .fetch_one(&mut **transaction)
            .await
            .map_err(CreateDealsTaskError::Sqlx)?;
            let segment_ids: Vec<String> = bin
                .snapshot_segments
                .iter()
                .map(|segment| segment.id.to_string())
                .collect();
            let aggregate_segment_size: i64 = bin
                .snapshot_segments
                .iter()
                .map(|segment| segment.size)
                .sum();

            let segment_ids_str = segment_ids.join(", ");
            let snapshot_ids: Vec<String> = sqlx::query!(
                r#"
                SELECT snapshot_id
                FROM snapshot_segment_associations
                WHERE segment_id IN ($1)
                "#,
                segment_ids_str
            )
            .fetch_all(&mut **transaction)
            .await
            .map_err(CreateDealsTaskError::Sqlx)?
            .into_iter()
            .map(|row| row.snapshot_id)
            .collect();

            // this will drop the old segment and, as a consequence, the snapshot_segment_association
            sqlx::query!(
                r#"
                DELETE FROM deals
                WHERE id IN (
                    SELECT deal_id
                    FROM snapshot_segments
                    WHERE id IN ($1)
                )
                "#,
                segment_ids_str
            )
            .execute(&mut **transaction)
            .await
            .map_err(CreateDealsTaskError::Sqlx)?;

            let new_segment_id = sqlx::query_scalar!(
                r#"
                INSERT INTO snapshot_segments (deal_id, size)
                VALUES ($1, $2)
                RETURNING id
                "#,
                new_deal_id,
                aggregate_segment_size,
            )
            .fetch_one(&mut **transaction)
            .await
            .map_err(CreateDealsTaskError::Sqlx)?;

            for snapshot_id in snapshot_ids {
                sqlx::query!(
                    r#"
                    INSERT INTO snapshot_segment_associations (snapshot_id, segment_id)
                    VALUES ($1, $2)
                    "#,
                    snapshot_id,
                    new_segment_id
                )
                .execute(&mut **transaction)
                .await
                .map_err(CreateDealsTaskError::Sqlx)?;
            }
        }
        Ok(())
    }
}
#[derive(sqlx::FromRow)]
struct SnapshotInfo {
    snapshot_id: String,
    block_count: Option<i64>,
    metadata_size: Option<i64>,
}

#[async_trait]
impl TaskLike for CreateDealsTask {
    const TASK_NAME: &'static str = "create_deals_task";

    type Error = CreateDealsTaskError;
    type Context = AppState;

    async fn run(&self, _task: CurrentTask, ctx: Self::Context) -> Result<(), Self::Error> {
        let mut transaction = ctx
            .database()
            .begin()
            .await
            .map_err(CreateDealsTaskError::Sqlx)?;

        let snapshot_info = sqlx::query_as!(
            SnapshotInfo,
            r#"
                SELECT snapshot_id, count(block_id) as block_count, m.metadata_size as metadata_size
                FROM snapshot_block_locations bls
                        JOIN snapshots s on bls.snapshot_id = s.id
                        JOIN metadata m on s.metadata_id = m.id
                WHERE s.id == $1 AND m.state == $2
                GROUP BY snapshot_id;
            "#,
            self.snapshot_id,
            MetadataState::Current
        )
        .fetch_one(&mut *transaction)
        .await
        .map_err(CreateDealsTaskError::Sqlx)?;

        let segment_size = snapshot_info.block_count.unwrap_or(0) * BLOCK_SIZE
            + snapshot_info.metadata_size.unwrap_or(0);
        let pending_snapshot_segments = self.split_into_segments(segment_size);

        if pending_snapshot_segments.len() > 1 {
            Self::create_multi_segment_deal(
                &mut transaction,
                &pending_snapshot_segments,
                &snapshot_info.snapshot_id,
            )
            .await?;
        } else {
            Self::aggregate_segments(&mut transaction, pending_snapshot_segments).await?;
        }

        transaction.commit().await?;

        Ok(())
    }
}

#[derive(Clone, Debug)]
struct Bin {
    snapshot_segments: Vec<SnapshotSegment>,
    free_space: i64,
}

impl Bin {
    fn new(capacity: i64) -> Self {
        Bin {
            snapshot_segments: Vec::new(),
            free_space: capacity,
        }
    }

    fn can_fit(&self, segment_size: i64) -> bool {
        self.free_space >= segment_size
    }

    fn add_snaphot_segment(&mut self, segment: SnapshotSegment) {
        self.free_space -= segment.size;
        self.snapshot_segments.push(segment);
    }
}

/// This function implements the Best Fit Decreasing (BFD) algorithm for bin packing.
///
/// The running time is O(nlogn) + O(n^2), where n is the number of items. It should be good enough for now.
fn best_fit_decreasing(snapshot_segments: Vec<SnapshotSegment>, bin_capacity: i64) -> Vec<Bin> {
    let mut sorted_segments = snapshot_segments;
    sorted_segments.sort_by(|a, b| b.size.cmp(&a.size));

    let mut bins: Vec<Bin> = Vec::new();

    for segment in sorted_segments {
        let segment_size = segment.size;
        if segment_size > bin_capacity {
            tracing::error!(segment_id = %segment.id, "got a segment larger than the bin_capicity");
            continue;
        }

        let mut best_fit_bin_index = None;
        let mut min_left = bin_capacity;

        for (i, bin) in bins.iter().enumerate() {
            if bin.can_fit(segment_size) && (bin.free_space - segment_size < min_left) {
                best_fit_bin_index = Some(i);
                min_left = bin.free_space - segment_size;
            }
        }

        match best_fit_bin_index {
            Some(index) => bins[index].add_snaphot_segment(segment),
            None => {
                let mut new_bin = Bin::new(bin_capacity);
                new_bin.add_snaphot_segment(segment);
                bins.push(new_bin);
            }
        }
    }

    bins
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use time::OffsetDateTime;

    use super::*;

    impl SnapshotSegment {
        pub fn new(id: String, size: i64, deal_id: String) -> Self {
            Self {
                id,
                size,
                deal_id,
                created_at: OffsetDateTime::now_utc(),
                updated_at: OffsetDateTime::now_utc(),
            }
        }
    }

    #[test]
    fn test_split_into_single_segment() {
        let deal_task = CreateDealsTask::new("snapshot_id".to_string());
        let segments = deal_task.split_into_segments(345);
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].size, 345);
    }

    #[test]
    fn test_split_into_multiple_segments() {
        let deal_task = CreateDealsTask::new("snapshot_id".to_string());
        let segments = deal_task.split_into_segments(MAX_SNAPSHOT_SEGMENT_SIZE * 2);
        assert_eq!(segments.len(), 2);
        assert!(segments.iter().all(|s| s.size == MAX_SNAPSHOT_SEGMENT_SIZE));
    }

    #[test]
    fn test_split_into_multiple_segments_with_left_over() {
        let deal_task = CreateDealsTask::new("snapshot_id".to_string());
        let segments = deal_task.split_into_segments(MAX_SNAPSHOT_SEGMENT_SIZE * 2 + 1);
        assert_eq!(segments.len(), 3);
        assert_eq!(segments[0].size, MAX_SNAPSHOT_SEGMENT_SIZE);
        assert_eq!(segments[1].size, MAX_SNAPSHOT_SEGMENT_SIZE);
        assert_eq!(segments[2].size, 1);
    }

    fn bins_are_disjoint_sets(bins: &Vec<Bin>) {
        let mut all_snapshots = HashSet::new();
        for bin in bins {
            let snapshot_segments: HashSet<_> = bin
                .snapshot_segments
                .iter()
                .map(|segment| segment.id.clone())
                .collect();
            assert!(all_snapshots.is_disjoint(&snapshot_segments));
            all_snapshots.extend(snapshot_segments);
        }
    }

    #[test]
    fn test_typical_case() {
        #[rustfmt::skip]
            let snapshot_segments = vec![
                SnapshotSegment::new("s1".to_string(), 10, "d1".to_string()),
                SnapshotSegment::new("s2".to_string(), 20, "d2".to_string()),
                SnapshotSegment::new("s3".to_string(), 5, "d3".to_string()),
            ];

        let bin_capacity = 32;
        let bins = best_fit_decreasing(snapshot_segments, bin_capacity);

        assert_eq!(bins.len(), 2);

        bins_are_disjoint_sets(&bins);
    }

    #[test]
    fn test_empty_snapshots() {
        let snapshot_segments = vec![];
        let bin_capacity = 32;

        let bins = best_fit_decreasing(snapshot_segments, bin_capacity);

        assert!(bins.is_empty());
    }

    #[test]
    fn test_single_large_item() {
        #[rustfmt::skip]
            let snapshot_segments = vec![
                SnapshotSegment::new("s1".to_string(), 32, "d1".to_string())
            ];
        let bin_capacity = 32;

        let bins = best_fit_decreasing(snapshot_segments, bin_capacity);

        assert_eq!(bins.len(), 1);
        assert_eq!(bins[0].snapshot_segments.len(), 1);
        assert_eq!(bins[0].free_space, 0);
    }

    #[test]
    fn test_item_larger_than_bin_capacity() {
        #[rustfmt::skip]
            let snapshot_segments = vec![
                SnapshotSegment::new("s1".to_string(), 40, "d1".to_string())
            ];
        let bin_capacity = 32;

        let bins = best_fit_decreasing(snapshot_segments, bin_capacity);

        assert_eq!(bins.len(), 0);
    }

    #[test]
    fn test_tightly_packed() {
        #[rustfmt::skip]
            let snapshot_segments = vec![
                SnapshotSegment::new("s1".to_string(), 2, "d1".to_string()),
                SnapshotSegment::new("s2".to_string(), 5, "d2".to_string()),
                SnapshotSegment::new("s3".to_string(), 4, "d3".to_string()),
                SnapshotSegment::new("s4".to_string(), 7, "d4".to_string()),
                SnapshotSegment::new("s5".to_string(), 1, "d5".to_string()),
                SnapshotSegment::new("s6".to_string(), 3, "d6".to_string()),
                SnapshotSegment::new("s7".to_string(), 8, "d7".to_string()),
            ];
        let bin_capacity = 10;

        let bins = best_fit_decreasing(snapshot_segments, bin_capacity);

        assert_eq!(bins.len(), 3);
        assert_eq!(bins[0].free_space, 0);
        assert_eq!(bins[1].free_space, 0);
        assert_eq!(bins[2].free_space, 0);
        assert_eq!(bins[0].snapshot_segments.len(), 2);
        assert_eq!(bins[1].snapshot_segments.len(), 2);
        assert_eq!(bins[2].snapshot_segments.len(), 3);
        bins_are_disjoint_sets(&bins);
    }
}
