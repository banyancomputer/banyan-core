use time::OffsetDateTime;

use crate::database::DatabaseConnection;

#[derive(Debug, sqlx::FromRow)]
pub struct StorageHostTotalConsumption {
    pub storage_host_id: String,
    pub storage_bytes: i64,
    pub slot: OffsetDateTime,
}

impl StorageHostTotalConsumption {
    pub async fn save(&self, db: &mut DatabaseConnection) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO storage_host_total_consumption
            (storage_host_id, storage_bytes, slot)
            VALUES ($1, $2, $3)
        ",
            self.storage_host_id,
            self.storage_bytes,
            self.slot,
        )
        .execute(&mut *db)
        .await?;
        Ok(())
    }

    pub async fn update(
        &self,
        db: &mut DatabaseConnection,
        storage_bytes: i64,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE storage_host_total_consumption SET storage_bytes = $1
            WHERE storage_host_id = $2 AND slot = $3",
            storage_bytes,
            self.storage_host_id,
            self.slot,
        )
        .execute(&mut *db)
        .await?;
        Ok(())
    }

    pub async fn find_by_slot_and_host(
        db: &mut DatabaseConnection,
        slot: OffsetDateTime,
        storage_host_id: &str,
    ) -> Result<Option<Self>, sqlx::Error> {
        let result = sqlx::query_as!(
            Self,
            "SELECT * FROM storage_host_total_consumption WHERE slot = $1 AND storage_host_id = $2",
            slot,
            storage_host_id,
        )
        .fetch_optional(&mut *db)
        .await?;

        Ok(result)
    }
}
