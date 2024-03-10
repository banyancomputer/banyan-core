#[derive(sqlx::FromRow)]
pub struct HotUsage {
    pub data_size: i32,
    pub meta_size: i32,
}

impl HotUsage {
    pub fn total(&self) -> i64 {
        self.data_size as i64 + self.meta_size as i64
    }
}
