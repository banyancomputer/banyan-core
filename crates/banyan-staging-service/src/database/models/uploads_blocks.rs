use time::OffsetDateTime;

#[derive(Debug, sqlx::FromRow)]
pub struct UploadsBlocks {
    pub upload_id: String,
    pub block_id: String,
    pub byte_offset: i64,
    pub associated_at: OffsetDateTime,
    pub pruned_at: Option<OffsetDateTime>,
}
