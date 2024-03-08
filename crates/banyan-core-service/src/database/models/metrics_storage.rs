use time::OffsetDateTime;

use crate::database::DatabaseConnection;

#[derive(Debug, sqlx::FromRow)]
pub struct MetricsStorage {
    pub user_id: String,
    pub hot_storage_bytes: i64,
    pub archival_storage_bytes: i64,
    pub slot: OffsetDateTime,
}
impl MetricsStorage {
    pub async fn save(&self, db: &mut DatabaseConnection) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO metrics_storage (user_id, hot_storage_bytes, archival_storage_bytes, slot)
        VALUES ($1, $2, $3, $4)
        ",
            self.user_id,
            self.hot_storage_bytes,
            self.archival_storage_bytes,
            self.slot,
        )
        .execute(&mut *db)
        .await?;
        Ok(())
    }

    pub async fn update(
        &self,
        db: &mut DatabaseConnection,
        new_hot_storage_bytes: i64,
        new_archival_storage_bytes: i64,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE metrics_storage SET hot_storage_bytes = $1, archival_storage_bytes = $2 WHERE user_id = $3 AND slot = $4",
            new_hot_storage_bytes,
            new_archival_storage_bytes,
            self.user_id,
            self.slot,
        )
        .execute(&mut *db)
        .await?;
        Ok(())
    }

    pub async fn find_by_slot_and_user(
        db: &mut DatabaseConnection,
        slot: OffsetDateTime,
        user_id: &str,
    ) -> Result<Option<Self>, sqlx::Error> {
        let result = sqlx::query_as!(
            Self,
            "SELECT * FROM metrics_storage WHERE slot = $1 AND user_id = $2",
            slot,
            user_id,
        )
        .fetch_optional(&mut *db)
        .await?;

        Ok(result)
    }
}
