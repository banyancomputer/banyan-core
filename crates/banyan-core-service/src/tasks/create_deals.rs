use async_trait::async_trait;
use banyan_task::{CurrentTask, TaskLike};
use serde::{Deserialize, Serialize};

use crate::app::AppState;
use crate::database::models::DealState;

const MAX_DEAL_SIZE: i64 = 32 * 1024 * 1024 * 1024; // 32GB

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

#[derive(sqlx::FromRow)]
struct SnapshotData {
    id: String,
    data_size: Option<i64>,
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

        let active_deal_state = DealState::Active.to_string();
        let pending_snapshots = sqlx::query_as!(
            SnapshotData,
            "SELECT s.id, m.data_size
                FROM snapshots as s
                LEFT JOIN deals d on s.deal_id = d.id
                JOIN metadata m on s.metadata_id = m.id
                WHERE (d.state = $1 OR s.deal_id IS NULL) AND m.data_size IS NOT NULL;",
            active_deal_state
        )
        .fetch_all(&mut *transaction)
        .await
        .map_err(CreateDealsTaskError::Sqlx)?;

        let bins = best_fit_decreasing(pending_snapshots, MAX_DEAL_SIZE);

        for bin in bins {
            let deal_id = sqlx::query_scalar!(
                r#"INSERT INTO deals (state) VALUES ($1) RETURNING id;"#,
                active_deal_state
            )
            .fetch_one(&mut *transaction)
            .await
            .map_err(CreateDealsTaskError::Sqlx)?;

            for (snapshot_id, _) in bin.snapshots {
                sqlx::query!(
                    "UPDATE snapshots SET deal_id = $1 WHERE id = $2 and deal_id",
                    deal_id,
                    snapshot_id
                )
                .execute(&mut *transaction)
                .await
                .map_err(CreateDealsTaskError::Sqlx)?;
            }
        }

        // clean up old snapshots were associated
        sqlx::query!(
            "DELETE FROM deals WHERE id NOT IN (SELECT deal_id FROM snapshots) AND state = $1",
            active_deal_state
        )
        .execute(&mut *transaction)
        .await
        .map_err(CreateDealsTaskError::Sqlx)?;

        transaction.commit().await?;

        Ok(())
    }
}

#[derive(Clone, Debug)]
struct Bin {
    snapshots: Vec<(String, i64)>,
    free_space: i64,
}

impl Bin {
    fn new(capacity: i64) -> Self {
        Bin {
            snapshots: Vec::new(),
            free_space: capacity,
        }
    }

    fn can_fit(&self, snapshot_size: i64) -> bool {
        self.free_space >= snapshot_size
    }

    fn add_snapshot(&mut self, snapshot_id: String, snapshot_size: i64) {
        self.snapshots.push((snapshot_id, snapshot_size));
        self.free_space -= snapshot_size;
    }
}

/// This function implements the Best Fit Decreasing (BFD) algorithm for bin packing.
///
/// The running time is O(n^2), where n is the number of items. It should be good enough for now.
fn best_fit_decreasing(snapshots: Vec<SnapshotData>, bin_capacity: i64) -> Vec<Bin> {
    let mut sorted_snapshots: Vec<_> = snapshots
        .into_iter()
        .filter_map(|s| s.data_size.map(|size| (s.id, size)))
        .collect();
    sorted_snapshots.sort_by(|a, b| b.1.cmp(&a.1));

    let mut bins: Vec<Bin> = Vec::new();

    for (snapshot_id, snapshot_size) in sorted_snapshots {
        if snapshot_size > bin_capacity {
            continue;
        }

        let mut best_fit_bin_index = None;
        let mut min_left = bin_capacity;

        for (i, bin) in bins.iter().enumerate() {
            if bin.can_fit(snapshot_size) && (bin.free_space - snapshot_size < min_left) {
                best_fit_bin_index = Some(i);
                min_left = bin.free_space - snapshot_size;
            }
        }

        match best_fit_bin_index {
            Some(index) => bins[index].add_snapshot(snapshot_id, snapshot_size),
            None => {
                let mut new_bin = Bin::new(bin_capacity);
                new_bin.add_snapshot(snapshot_id, snapshot_size);
                bins.push(new_bin);
            }
        }
    }

    bins
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    fn bins_are_disjoint_sets(bins: &Vec<Bin>) {
        let mut all_snapshots = HashSet::new();
        for bin in &bins {
            let snapshots: HashSet<_> = bin.snapshots.iter().map(|(id, size)| id).collect();
            assert!(all_snapshots.is_disjoint(&snapshots));
            all_snapshots.extend(snapshots);
        }
    }

    #[test]
    fn test_typical_case() {
        #[rustfmt::skip]
        let snapshots = vec![
            SnapshotData { id: "s1".to_string(), data_size: Some(10) },
            SnapshotData { id: "s2".to_string(), data_size: Some(20) },
            SnapshotData { id: "s3".to_string(), data_size: Some(5) },
        ];

        let bin_capacity = 32;
        let bins = best_fit_decreasing(snapshots, bin_capacity);

        assert_eq!(bins.len(), 2);

        bins_are_disjoint_sets(&bins);
    }

    #[test]
    fn test_empty_snapshots() {
        let snapshots = vec![];
        let bin_capacity = 32;

        let bins = best_fit_decreasing(snapshots, bin_capacity);

        assert!(bins.is_empty());
    }

    #[test]
    fn test_single_large_item() {
        #[rustfmt::skip]
        let snapshots = vec![
            SnapshotData {  id: "s1".to_string(), data_size: Some(32) }
        ];
        let bin_capacity = 32;

        let bins = best_fit_decreasing(snapshots, bin_capacity);

        assert_eq!(bins.len(), 1);
        assert_eq!(bins[0].snapshots.len(), 1);
        assert_eq!(bins[0].free_space, 0);
    }

    #[test]
    fn test_item_larger_than_bin_capacity() {
        #[rustfmt::skip]
        let snapshots = vec![
            SnapshotData {  id: "s1".to_string(), data_size: Some(40) }
        ];
        let bin_capacity = 32;

        let bins = best_fit_decreasing(snapshots, bin_capacity);

        assert_eq!(bins.len(), 0);
    }

    #[test]
    fn test_tightly_packed() {
        #[rustfmt::skip]
        let snapshots = vec![
            SnapshotData { id: "s1".to_string(), data_size: Some(2) },
            SnapshotData { id: "s2".to_string(), data_size: Some(5) },
            SnapshotData { id: "s3".to_string(), data_size: Some(4) },
            SnapshotData { id: "s4".to_string(), data_size: Some(7) },
            SnapshotData { id: "s5".to_string(), data_size: Some(1) },
            SnapshotData { id: "s6".to_string(), data_size: Some(3) },
            SnapshotData { id: "s7".to_string(), data_size: Some(8) },
        ];
        let bin_capacity = 10;

        let bins = best_fit_decreasing(snapshots, bin_capacity);

        assert_eq!(bins.len(), 3);
        assert_eq!(bins[0].free_space, 0);
        assert_eq!(bins[1].free_space, 0);
        assert_eq!(bins[2].free_space, 0);
        assert_eq!(bins[0].snapshots.len(), 2);
        assert_eq!(bins[1].snapshots.len(), 2);
        assert_eq!(bins[2].snapshots.len(), 3);
        bins_are_disjoint_sets(&bins);
    }
}
