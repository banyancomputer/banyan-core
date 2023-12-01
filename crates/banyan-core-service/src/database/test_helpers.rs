use sqlx::sqlite::SqlitePoolOptions;

use crate::database::models::{BucketType, MetadataState, StorageClass};
use crate::database::Database;

pub(crate) async fn create_hot_bucket(
    database: &Database,
    user_id: &str,
    name: &str,
) -> Result<String, sqlx::Error> {
    sqlx::query_scalar!(
        r#"INSERT INTO
                buckets (user_id, name, type, storage_class)
                VALUES ($1, $2, $3, $4)
                RETURNING id;"#,
        user_id,
        name,
        BucketType::Interactive,
        StorageClass::Hot,
    )
    .fetch_one(database)
    .await
}

pub(crate) async fn create_metadata(
    database: &Database,
    bucket_id: &str,
    root_cid: &str,
    metadata_cid: &str,
    expected_data_size: i64,
    state: MetadataState,
) -> Result<String, sqlx::Error> {
    sqlx::query_scalar!(
        r#"INSERT INTO
                metadata (bucket_id, root_cid, metadata_cid, expected_data_size, state)
                VALUES ($1, $2, $3, $4, $5)
                RETURNING id;"#,
        bucket_id,
        root_cid,
        metadata_cid,
        expected_data_size,
        state,
    )
    .fetch_one(database)
    .await
}

pub(crate) async fn create_user(
    database: &Database,
    email: &str,
    display_name: &str,
) -> Result<String, sqlx::Error> {
    sqlx::query_scalar!(
        r#"INSERT INTO
                users (email, verified_email, display_name)
                VALUES ($1, true, $2)
                RETURNING id;"#,
        email,
        display_name,
    )
    .fetch_one(database)
    .await
}

pub(crate) async fn setup_database() -> Database {
    let pool = SqlitePoolOptions::new()
        .connect("sqlite::memory:")
        .await
        .expect("Failed to connect to the database");

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("failed to run migrations");

    pool
}
