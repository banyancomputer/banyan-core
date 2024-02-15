use ecdsa::signature::rand_core::block::BlockRngCore;
use time::OffsetDateTime;

use crate::database::Database;

/// The triple of these attributes make up the unique association ID for the `block_locations`
/// table. This structure is appropriate to use whenever one or more of these rows needs to be
/// uniquely identified without the associated metadata on the link.
#[derive(Debug, sqlx::FromRow)]
pub struct MinimalBlockLocation {
    pub block_id: String,
    pub metadata_id: String,
    pub storage_host_id: String,
}

#[derive(Debug, sqlx::FromRow)]
pub struct BlockLocation {
    pub block_id: String,
    pub metadata_id: String,
    pub storage_host_id: String,
    pub associated_at: Option<OffsetDateTime>,
    pub pruned_at: Option<OffsetDateTime>,
    pub expired_at: Option<OffsetDateTime>,
}
impl BlockLocation {
    pub async fn get_all_for_host(
        db: &Database,
        storage_host_id: &str,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as!(
            Self,
            "SELECT * FROM block_locations WHERE storage_host_id = $1 and pruned_at = NULL and expired_at = NULL",
            storage_host_id
        )
        .fetch_all(db)
        .await
    }
}
