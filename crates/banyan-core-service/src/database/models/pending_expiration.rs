use crate::database::DatabaseConnection;

/// This table represents blocks that have been marked for expiration by an upload, it does not
/// represent the blocks contained within a particular metadata. This association is present to
/// delay when we mark blocks as expired (wait until the metadata becomes current).
pub struct PendingExpiration;

impl PendingExpiration {
    pub async fn record_pending_block_expirations(
        _conn: &mut DatabaseConnection,
        _metadata_id: &str,
        _block_cids: impl IntoIterator<Item = &str>,
    ) -> Result<(), sqlx::Error> {
        todo!()
    }
}
