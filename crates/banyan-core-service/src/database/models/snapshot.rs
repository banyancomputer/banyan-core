use time::OffsetDateTime;

use crate::database::models::{ExplicitBigInt, SnapshotState};
use crate::database::DatabaseConnection;

#[derive(Debug, sqlx::FromRow)]
pub struct Snapshot {
    pub id: String,
    pub bucket_id: String,
    pub metadata_id: String,
    pub state: SnapshotState,
    pub size: Option<i64>,
    pub created_at: OffsetDateTime,
}

impl Snapshot {
    pub async fn total_usage_for_user(
        conn: &mut DatabaseConnection,
        user_id: &str,
    ) -> Result<i64, sqlx::Error> {
        let result = sqlx::query_as!(
            ExplicitBigInt,
            "SELECT COALESCE(SUM(COALESCE(snapshots.size, 0)), 0) AS big_int
            FROM snapshots
                JOIN main.metadata m ON m.id = snapshots.metadata_id
                JOIN main.buckets b ON b.id = m.bucket_id
            WHERE user_id = $1
                AND b.deleted_at IS NULL
                AND m.state IN ('current', 'outdated', 'pending')
                AND snapshots.state != $2;",
            user_id,
            SnapshotState::Error
        )
        .fetch_optional(&mut *conn)
        .await?;

        match result {
            Some(result) => Ok(result.big_int),
            None => Ok(0),
        }
    }
}
