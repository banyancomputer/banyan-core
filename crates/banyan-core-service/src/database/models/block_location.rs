use sqlx::sqlite::SqliteQueryResult;

use crate::database::models::BlockLocationState;
use crate::database::{Database, DatabaseConnection, BIND_LIMIT};

/// The triple of these attributes make up the unique association ID for the `block_locations`
/// table. This structure is appropriate to use whenever one or more of these rows needs to be
/// uniquely identified without the associated metadata on the link.
#[derive(Debug, sqlx::FromRow)]
pub struct MinimalBlockLocation {
    pub block_id: String,
    pub metadata_id: String,
    pub state: BlockLocationState,
    pub storage_host_id: String,
}

impl MinimalBlockLocation {
    pub async fn delete_blocks_for_host(
        conn: &mut DatabaseConnection,
        block_ids: &[String],
        storage_host_id: &str,
    ) -> Result<Vec<SqliteQueryResult>, sqlx::Error> {
        let mut affected = Vec::new();
        for cid_chunk in block_ids.chunks(BIND_LIMIT) {
            let mut block_id_builder =
                sqlx::QueryBuilder::new("DELETE FROM block_locations WHERE storage_host_id = ");
            block_id_builder.push_bind(storage_host_id);
            block_id_builder.push(" AND block_locations.block_id IN (");

            let mut separated_values = block_id_builder.separated(", ");
            for cid in cid_chunk {
                separated_values.push_bind(cid);
            }

            block_id_builder.push(");");

            let res = block_id_builder.build().execute(&mut *conn).await?;
            affected.push(res);
        }

        Ok(affected)
    }

    pub async fn update_state(
        db: &Database,
        block_ids: &[String],
        new_state: BlockLocationState,
    ) -> Result<Vec<SqliteQueryResult>, sqlx::Error> {
        let mut affected = Vec::new();
        for cid_chunk in block_ids.chunks(BIND_LIMIT) {
            let mut block_id_builder =
                sqlx::QueryBuilder::new("UPDATE block_locations SET state = ");
            block_id_builder.push_bind(new_state.to_string());
            block_id_builder.push(" WHERE block_id IN (");
            let mut separated_values = block_id_builder.separated(", ");
            for cid in cid_chunk {
                separated_values.push_bind(cid);
            }

            block_id_builder.push(");");

            let res = block_id_builder.build().execute(db).await?;
            affected.push(res);
        }

        Ok(affected)
    }

    pub async fn save(&self, db: &mut DatabaseConnection) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO block_locations
            (block_id, metadata_id, storage_host_id, state)
            VALUES ($1, $2, $3, $4);",
            self.block_id,
            self.metadata_id,
            self.storage_host_id,
            self.state,
        )
        .execute(&mut *db)
        .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::database::models::{BlockLocationState, MetadataState, MinimalBlockLocation};
    use crate::database::test_helpers::{
        associate_blocks, create_blocks, create_storage_host, data_generator, generate_cids,
        normalize_cids, sample_bucket, sample_metadata, sample_user, setup_database,
    };
    use crate::database::Database;
    impl MinimalBlockLocation {
        pub async fn get_all(pool: &Database) -> Result<Vec<MinimalBlockLocation>, sqlx::Error> {
            sqlx::query_as!(
                MinimalBlockLocation,
                "SELECT block_id, metadata_id, storage_host_id, state FROM block_locations;"
            )
            .fetch_all(pool)
            .await
        }
    }

    #[tokio::test]
    async fn test_create_snapshot_works() {
        let db = setup_database().await;
        let mut conn = db.acquire().await.expect("connection");

        let user_id = sample_user(&mut conn, "test@example.com").await;
        let bucket_id = sample_bucket(&mut conn, &user_id).await;
        let metadata_id = sample_metadata(&mut conn, &bucket_id, 1, MetadataState::Current).await;
        let prim_storage_host_id =
            create_storage_host(&mut conn, "Diskz", "https://127.0.0.1:8001/", 1_000_000).await;
        let initial_cids: Vec<_> = normalize_cids(generate_cids(data_generator(0..3))).collect();
        let block_ids = create_blocks(&mut conn, initial_cids.iter().map(String::as_str)).await;
        associate_blocks(
            &mut conn,
            &metadata_id,
            &prim_storage_host_id,
            block_ids.iter().map(String::as_str),
        )
        .await;

        let new_blocks = MinimalBlockLocation::get_all(&db)
            .await
            .expect("get all block locations");
        assert_eq!(new_blocks.len(), 3);
        for block_location in new_blocks {
            assert_eq!(block_location.state, BlockLocationState::Stable);
        }
    }
}
