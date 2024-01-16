#[derive(sqlx::FromRow, Debug)]
pub struct BlockDetails {
    pub id: String,
    pub length: i32,
    pub car_offset: Option<i32>,
    pub platform_id: String,
    pub metadata_id: String,
}
