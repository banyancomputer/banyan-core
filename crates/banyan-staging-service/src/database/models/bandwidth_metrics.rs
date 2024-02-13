use time::OffsetDateTime;

use crate::database::Database;

#[derive(sqlx::FromRow, Debug)]
pub struct BandwidthMetrics {
    pub user_id: String,
    pub ingress: i64,
    pub egress: i64,
}
impl BandwidthMetrics {
    pub async fn save(&self, db: &Database, created_at: OffsetDateTime) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"INSERT INTO bandwidth_metrics (user_id, ingress, egress, created_at) VALUES ($1, $2, $3, $4);"#,
             self.user_id,
            self.ingress,
            self.egress,
            created_at,
        )
        .execute(db)
        .await
        .map(|_| ())
    }
}
