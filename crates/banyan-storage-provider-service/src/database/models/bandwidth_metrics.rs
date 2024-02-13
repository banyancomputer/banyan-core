use time::OffsetDateTime;

use crate::database::Database;

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct BandwidthMetrics {
    pub user_id: String,
    pub ingress: i64,
    pub egress: i64,
    pub created_at: OffsetDateTime,
}
impl BandwidthMetrics {
    pub async fn save(&self, db: &Database) -> Result<(), sqlx::Error> {
        let created_at = self.created_at;
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
