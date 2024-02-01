#[derive(sqlx::FromRow, Debug)]
pub struct BlockDetails {
    pub id: String,
    pub length: i64,
    pub car_offset: Option<i64>,
    pub base_path: String,
    pub platform_id: String,
}
