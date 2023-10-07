use time::OffsetDateTime;

use crate::database::models::MetadataState;

#[derive(sqlx::FromRow)]
pub struct Metadata {
    pub id: String,
    pub bucket_id: String,

    // todo: should be associated to users table, requires a lot of rework

    pub root_cid: String,
    pub metadata_cid: String,

    pub expected_data_size: i64,
    pub data_size: Option<i64>,

    pub metadata_size: Option<i64>,
    pub metadata_hash: Option<String>,

    pub state: MetadataState,

    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}
