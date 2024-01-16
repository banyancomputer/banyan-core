#[derive(sqlx::FromRow, Debug)]
pub struct BlockDetails {
    pub id: String,
    pub platform_id: String,

    pub metadata_id: String,
    pub byte_offset: i32,
    pub length: i32,
}
