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
        12_123_100,
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

pub(crate) async fn current_metadata(db: &Database, bucket_id: &str, counter: usize) -> String {
    sample_metadata(db, bucket_id, counter, MetadataState::Current).await
}

pub(crate) async fn pending_metadata(db: &Database, bucket_id: &str, counter: usize) -> String {
    sample_metadata(db, bucket_id, counter, MetadataState::Pending).await
}

pub(crate) async fn sample_bucket(db: &Database) -> String {
    let user_id = sample_user(&db).await;

    create_hot_bucket(&db, &user_id, "Habernero")
        .await
        .expect("bucket creation")
}

pub(crate) async fn sample_metadata(
    db: &Database,
    bucket_id: &str,
    counter: usize,
    state: MetadataState,
) -> String {
    let root_cid = format!("root-cid-{}", counter);
    let metadata_cid = format!("metadata-cid-{}", counter);

    create_metadata(&db, &bucket_id, &root_cid, &metadata_cid, state)
        .await
        .expect("current metadata creation")
}

pub(crate) async fn sample_user(db: &Database) -> String {
    create_user(&db, "francesca@sample.users.org", "Francesca Tester")
        .await
        .expect("user creation")
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
