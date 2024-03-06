use async_trait::async_trait;
use banyan_task::{CurrentTask, TaskLike};
use serde::{Deserialize, Serialize};
use sqlx::{QueryBuilder, Sqlite, Transaction};

use crate::app::AppState;
use crate::database::models::{DealState, MetadataState, SnapshotSegment};

const MAX_SNAPSHOT_SEGMENT_SIZE: i64 = 32 * 1024 * 1024 * 1024; // 32GiB
                                                                //
pub const BLOCK_SIZE: i64 = 262144; // 256 KiB
                                    //
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
        snapshot_id: &str,
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

            let mut snapshot_ids =
                Self::retrieve_snapshots(transaction, segment_ids.clone()).await?;
            snapshot_ids.push(snapshot_id.to_string());
            Self::delete_deals(transaction, segment_ids).await?;

            let new_deal_id = sqlx::query_scalar!(
                r#" INSERT INTO deals (state) VALUES ($1) RETURNING id;"#,
                DealState::Active,
            )
            .fetch_one(&mut **transaction)
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

    async fn retrieve_snapshots(
        transaction: &mut Transaction<'_, Sqlite>,
        segment_ids: Vec<String>,
    ) -> Result<Vec<String>, <CreateDealsTask as TaskLike>::Error> {
        let mut snapshots_builder = sqlx::QueryBuilder::new(
            "SELECT DISTINCT snapshot_id FROM snapshot_segment_associations WHERE segment_id IN (",
        );

        let mut segment_iterator = segment_ids.iter().peekable();
        while let Some(segment_id) = segment_iterator.next() {
            snapshots_builder.push_bind(segment_id);

            if segment_iterator.peek().is_some() {
                snapshots_builder.push(", ");
            }
        }
        snapshots_builder.push(");");

        let snapshot_ids = snapshots_builder
            .build_query_scalar()
            .persistent(false)
            .fetch_all(&mut **transaction)
            .await
            .map_err(CreateDealsTaskError::Sqlx)?;
        Ok(snapshot_ids)
    }

    async fn delete_deals(
        transaction: &mut Transaction<'_, Sqlite>,
        segment_ids: Vec<String>,
    ) -> Result<(), <CreateDealsTask as TaskLike>::Error> {
        // this will drop the old segment and, as a consequence, the snapshot_segment_association
        let mut deals_builder = sqlx::QueryBuilder::new(
            "DELETE FROM deals WHERE id IN (SELECT deal_id FROM snapshot_segments WHERE id IN (",
        );
        let mut segment_iterator = segment_ids.iter().peekable();
        while let Some(segment_id) = segment_iterator.next() {
            deals_builder.push_bind(segment_id);

            if segment_iterator.peek().is_some() {
                deals_builder.push(", ");
            }
        }
        deals_builder.push(")");
        deals_builder.push(");");
        deals_builder.build().execute(&mut **transaction).await?;
        Ok(())
    }

    #[allow(dead_code)]
    async fn return_tokens(
        transaction: &mut Transaction<'_, Sqlite>,
        snapshot_ids: &Vec<String>,
    ) -> Result<(), <CreateDealsTask as TaskLike>::Error> {
        let mut builder = QueryBuilder::new(
            r#"
                UPDATE users AS u
                SET u.consumed_tokens = u.consumed_tokens - (
                    SELECT u.tokens_used
                    FROM snapshots AS s
                    WHERE s.user_id = u.id
                    AND s.id IN (
            "#,
        );

        let mut separated = builder.separated(", ");
        for snapshot_id in snapshot_ids {
            separated.push_bind(snapshot_id);
        }

        builder.push(")");
        builder.push(");");
        builder.build().execute(&mut **transaction).await?;

        Ok(())
    }
}
#[derive(Debug, sqlx::FromRow)]
struct SnapshotInfo {
    snapshot_id: String,
    snapshot_size: Option<i64>,
    metadata_size: Option<i64>,
}

#[async_trait]
impl TaskLike for CreateDealsTask {
    const TASK_NAME: &'static str = "create_deals_task";

    type Error = CreateDealsTaskError;
    type Context = AppState;

    async fn run(&self, _task: CurrentTask, ctx: Self::Context) -> Result<(), Self::Error> {
        let mut transaction = ctx.database().begin().await?;

        let snapshot_info = sqlx::query_as!(
            SnapshotInfo,
            r#"
                SELECT snapshot_id, s.size as snapshot_size, m.metadata_size as metadata_size
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
        .await?;

        let segment_size =
            snapshot_info.snapshot_size.unwrap_or(0) + snapshot_info.metadata_size.unwrap_or(0);
        let pending_snapshot_segments = self.split_into_segments(segment_size);

        if pending_snapshot_segments.len() > 1 {
            Self::create_multi_segment_deal(
                &mut transaction,
                &pending_snapshot_segments,
                &snapshot_info.snapshot_id,
            )
            .await?;
        } else {
            Self::aggregate_segments(
                &mut transaction,
                pending_snapshot_segments,
                &snapshot_info.snapshot_id,
            )
            .await?;
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

    use banyan_task::tests::default_current_task;
    use banyan_task::TaskLike;
    use time::OffsetDateTime;

    use crate::app::mock_app_state;
    use crate::database::models::{
        Deal, DealState, MetadataState, Snapshot, SnapshotSegment, SnapshotState,
    };
    use crate::database::test_helpers::{
        create_blocks, create_deal, create_snapshot, create_snapshot_block_locations,
        data_generator, generate_cids, normalize_cids, sample_bucket, sample_metadata, sample_user,
        setup_database,
    };
    use crate::database::{Database, DatabaseConnection};
    use crate::tasks::create_deals::{best_fit_decreasing, Bin, MAX_SNAPSHOT_SEGMENT_SIZE};
    use crate::tasks::{CreateDealsTask, BLOCK_SIZE};

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

        pub(crate) async fn get_all(conn: &Database) -> Vec<SnapshotSegment> {
            sqlx::query_as!(SnapshotSegment, "SELECT * FROM snapshot_segments;")
                .fetch_all(conn)
                .await
                .expect("fetch snapshot segments")
        }
    }

    impl Snapshot {
        pub(crate) async fn get_all(conn: &Database) -> Vec<Snapshot> {
            sqlx::query_as!(Snapshot, "SELECT * FROM snapshots;")
                .fetch_all(conn)
                .await
                .expect("fetch snapshots")
        }
    }

    impl Deal {
        pub(crate) async fn get_all(conn: &Database) -> Vec<Deal> {
            sqlx::query_as!(
                Deal,
                "SELECT id, state, 0 as size, accepted_at, accepted_by FROM deals;"
            )
            .fetch_all(conn)
            .await
            .expect("fetch deals")
        }
    }

    async fn count_segment_associations(conn: &mut DatabaseConnection) -> i32 {
        sqlx::query_scalar!("SELECT COUNT(*) FROM snapshot_segment_associations;")
            .fetch_one(&mut *conn)
            .await
            .expect("fetch snapshot association size")
    }

    #[tokio::test]
    async fn test_single_snapshot_deal_works() {
        let db = setup_database().await;
        let state = mock_app_state(db.clone());
        let mut conn = db.acquire().await.expect("connection");
        let user_id = sample_user(&mut conn, "user1@domain.tld").await;
        let bucket_id = sample_bucket(&mut conn, &user_id).await;
        let metadata_id = sample_metadata(&mut conn, &bucket_id, 1, MetadataState::Current).await;
        let initial_cids: Vec<_> = normalize_cids(generate_cids(data_generator(0..4))).collect();
        let block_ids = create_blocks(&mut conn, initial_cids.iter().map(String::as_str)).await;

        let snapshot_id = create_snapshot(
            &mut conn,
            &metadata_id,
            SnapshotState::Completed,
            Some(block_ids.len() as i64 * BLOCK_SIZE),
        )
        .await;
        create_snapshot_block_locations(&mut conn, &snapshot_id, block_ids).await;

        let task = CreateDealsTask::new(snapshot_id);
        let res = task.run(default_current_task(), state.0).await;

        assert!(res.is_ok());
        let all_snapshots = Snapshot::get_all(&db).await;
        let all_snapshot_segments = SnapshotSegment::get_all(&db).await;
        let all_deals = Deal::get_all(&db).await;
        let number_of_segment_associations = count_segment_associations(&mut conn).await;
        assert_eq!(all_deals.len(), 1);
        assert_eq!(all_snapshots.len(), 1);
        assert_eq!(all_snapshot_segments.len(), 1);
        assert_eq!(all_snapshot_segments[0].size, 1048576);
        assert_eq!(
            all_snapshots
                .iter()
                .map(|s| s.size.unwrap_or(0))
                .sum::<i64>(),
            1048576
        );
        assert_eq!(number_of_segment_associations, 1);
    }

    #[tokio::test]
    async fn test_deal_creation_for_two_different_users() {
        let db = setup_database().await;
        let state = mock_app_state(db.clone());
        let mut conn = db.acquire().await.expect("connection");

        create_deal(&mut conn, DealState::Active, Some(2 * BLOCK_SIZE), None)
            .await
            .unwrap();

        let user_id = sample_user(&mut conn, "user1@domain.tld").await;
        let bucket_id = sample_bucket(&mut conn, &user_id).await;
        let metadata_id = sample_metadata(&mut conn, &bucket_id, 1, MetadataState::Current).await;
        let snapshot_id = create_snapshot(
            &mut conn,
            &metadata_id,
            SnapshotState::Completed,
            Some(BLOCK_SIZE),
        )
        .await;
        let initial_cids: Vec<_> = normalize_cids(generate_cids(data_generator(0..4))).collect();
        let block_ids = create_blocks(&mut conn, initial_cids.iter().map(String::as_str)).await;
        create_snapshot_block_locations(&mut conn, &snapshot_id, block_ids).await;

        let task = CreateDealsTask::new(snapshot_id);
        let res = task.run(default_current_task(), state.0).await;

        assert!(res.is_ok());
        let all_snapshots = Snapshot::get_all(&db).await;
        let all_snapshot_segments = SnapshotSegment::get_all(&db).await;
        let all_deals = Deal::get_all(&db).await;
        let number_of_segment_associations = count_segment_associations(&mut conn).await;
        assert_eq!(all_deals.len(), 1);
        assert_eq!(all_snapshots.len(), 2);
        assert_eq!(all_snapshot_segments.len(), 1);
        assert_eq!(all_snapshot_segments[0].size, 1310720);
        assert_eq!(
            all_snapshots
                .iter()
                .map(|s| s.size.unwrap_or(0))
                .sum::<i64>(),
            1310720
        );
        assert_eq!(number_of_segment_associations, 2);
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
