use time::OffsetDateTime;

use crate::database::DatabaseConnection;

#[derive(Debug, sqlx::FromRow)]
pub struct MetricsStorage {
    pub user_id: String,
    pub hot_storage_bytes: i64,
    pub archival_storage_bytes: i64,
    pub storage_host_id: String,
    pub slot: OffsetDateTime,
}
impl MetricsStorage {
    pub async fn save(&self, db: &mut DatabaseConnection) -> Result<(), sqlx::Error> {
        sqlx::query!(
        "INSERT INTO metrics_storage (user_id, hot_storage_bytes, archival_storage_bytes, storage_host_id, slot)
        VALUES ($1, $2, $3, $4, $5)
        ",
        self.user_id,
        self.hot_storage_bytes,
        self.archival_storage_bytes,
        self.storage_host_id,
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
            "UPDATE metrics_storage SET hot_storage_bytes = $1, archival_storage_bytes = $2 WHERE user_id = $3 AND storage_host_id = $4 AND slot = $5",
            new_hot_storage_bytes,
            new_archival_storage_bytes,
            self.user_id,
            self.storage_host_id,
            self.slot,
        )
        .execute(&mut *db)
        .await?;
        Ok(())
    }

    pub async fn find_by_slot_user_and_storage_host(
        db: &mut DatabaseConnection,
        slot: OffsetDateTime,
        user_id: String,
        storage_host_id: String,
    ) -> Result<Option<Self>, sqlx::Error> {
        let result = sqlx::query_as!(
            Self,
            "SELECT * FROM metrics_storage WHERE slot = $1 AND user_id = $2 AND storage_host_id = $3",
            slot,
            user_id,
            storage_host_id
        )
        .fetch_optional(&mut *db)
        .await?;

        Ok(result)
    }
}
