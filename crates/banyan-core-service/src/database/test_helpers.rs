use sqlx::sqlite::SqlitePoolOptions;

use crate::database::models::{BucketType, DealState, MetadataState, StorageClass};
use crate::database::Database;

pub(crate) async fn create_storage_hosts(
    database: &Database,
    host_url: &str,
    host_name: &str,
) -> Result<String, sqlx::Error> {
    let host_url = host_url.to_string();
    let host_name = host_name.to_string();
    sqlx::query_scalar!(
            r#"INSERT INTO storage_hosts (id, name, url, fingerprint, pem, used_storage, available_storage)
            VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id;"#,
            host_name,
            host_url,
            "fingerprint_1",
            "pem_1",
            "hello.com",
            0,
            0
        )
    .fetch_one(database)
    .await
}

pub(crate) async fn create_deal(
    database: &Database,
    deal_state: DealState,
    accepted_by: Option<String>,
) -> Result<String, sqlx::Error> {
    let deal_state = deal_state.to_string();
    match accepted_by {
        Some(accepted_by) => {
            sqlx::query_scalar!(
                r#"INSERT INTO deals (state, accepted_by, accepted_at) VALUES ($1, $2, datetime('now')) RETURNING id;"#,
                deal_state,
                accepted_by
            )
            .fetch_one(database)
            .await
        },
        None => {
            sqlx::query_scalar!(
                r#"INSERT INTO deals (state) VALUES ($1) RETURNING id;"#,
                deal_state
            )
            .fetch_one(database)
            .await
        }
    }
}

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
    let user_id = sample_user(db, 1).await;

    create_hot_bucket(db, &user_id, "Habernero")
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

    create_metadata(db, bucket_id, &root_cid, &metadata_cid, state)
        .await
        .expect("current metadata creation")
}

pub(crate) async fn assert_metadata_state(
    db: &Database,
    metadata_id: &str,
    expected_state: MetadataState,
) {
    let found_state = sqlx::query_scalar!(
        r#"SELECT state as 'state: MetadataState' FROM metadata WHERE id = $1;"#,
        metadata_id,
    )
    .fetch_one(db)
    .await
    .expect("metadata existence");

    assert_eq!(found_state, expected_state);
}

pub(crate) async fn sample_user(db: &Database, counter: usize) -> String {
    create_user(
        db,
        &format!("jessica_{counter}@sample.users.org"),
        &format!("Jessica {counter} Tester"),
    )
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
