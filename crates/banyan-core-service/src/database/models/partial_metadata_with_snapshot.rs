use uuid::Uuid;

use crate::database::models::MetadataState;
use crate::database::types::PrecisionTimestamp;
use crate::database::Database;

#[derive(sqlx::FromRow)]
pub struct PartialMetadataWithSnapshot {
    pub id: String,

    pub root_cid: String,
    pub metadata_cid: String,
    pub data_size: Option<i64>,

    pub state: MetadataState,

    pub created_at: PrecisionTimestamp,
    pub updated_at: PrecisionTimestamp,

    pub snapshot_id: Option<String>,
}

impl PartialMetadataWithSnapshot {
    pub async fn all(database: &Database, user_id: String) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as!(
            Self,
            r#"SELECT
                    m.id, m.root_cid, m.metadata_cid,
                    COALESCE(m.data_size, m.expected_data_size) as data_size,
                    m.state as 'state: MetadataState', m.created_at, m.updated_at, s.id as snapshot_id
                FROM metadata m
                    JOIN buckets b ON m.bucket_id = b.id
                    LEFT JOIN snapshots s ON s.metadata_id = m.id
                    WHERE m.state NOT IN ('upload_failed', 'deleted')
                          AND b.user_id = $1;"#,
            user_id,
        )
        .fetch_all(database)
        .await
    }

    pub async fn locate_current(
        database: &Database,
        user_id: String,
        bucket_id: Uuid,
    ) -> Result<Option<Self>, sqlx::Error> {
        let bucket_id = bucket_id.to_string();

        let query_result = sqlx::query_as!(
            Self,
            r#"SELECT
                    m.id, m.root_cid, m.metadata_cid,
                    COALESCE(m.data_size, m.expected_data_size) as data_size,
                    m.state as 'state: MetadataState', m.created_at, m.updated_at, s.id as snapshot_id
                FROM metadata m
                    JOIN buckets b ON m.bucket_id = b.id
                    LEFT JOIN snapshots s ON s.metadata_id = m.id
                    WHERE m.state = 'current' AND b.user_id = $1 AND b.id = $2;"#,
            user_id,
            bucket_id,
        )
        .fetch_one(database)
        .await;

        match query_result {
            Ok(pmws) => Ok(Some(pmws)),
            Err(sqlx::Error::RowNotFound) => Ok(None),
            Err(err) => Err(err),
        }
    }

    pub async fn locate_specific(
        database: &Database,
        user_id: String,
        bucket_id: Uuid,
        metadata_id: Uuid,
    ) -> Result<Option<Self>, sqlx::Error> {
        let bucket_id = bucket_id.to_string();
        let metadata_id = metadata_id.to_string();

        let query_result = sqlx::query_as!(
            Self,
            r#"SELECT
                    m.id, m.root_cid, m.metadata_cid,
                    COALESCE(m.data_size, m.expected_data_size) as data_size,
                    m.state as 'state: MetadataState', m.created_at, m.updated_at, s.id as snapshot_id
                FROM metadata m
                    JOIN buckets b ON m.bucket_id = b.id
                    LEFT JOIN snapshots s ON s.metadata_id = m.id
                    WHERE m.id = $1 AND b.user_id = $2 AND b.id = $3;"#,
            metadata_id,
            user_id,
            bucket_id,
        )
        .fetch_one(database)
        .await;

        match query_result {
            Ok(pmws) => Ok(Some(pmws)),
            Err(sqlx::Error::RowNotFound) => Ok(None),
            Err(err) => Err(err),
        }
    }
}
