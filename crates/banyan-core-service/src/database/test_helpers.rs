use sqlx::sqlite::{SqlitePoolOptions, SqliteQueryResult};

use crate::database::models::{BucketType, DealState, MetadataState, SnapshotState, StorageClass};
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
    size: Option<i64>,
    accepted_by: Option<String>,
) -> Result<String, sqlx::Error> {
    let user_id = sample_user(database).await;
    let bucket_id = create_hot_bucket(database, user_id.as_str(), "test_bucket")
        .await
        .unwrap();
    let metadata_id = create_metadata(
        database,
        bucket_id.as_str(),
        format!("root-cid-{}", bucket_id).as_str(),
        format!("metadata-cid-{}", bucket_id).as_str(),
        MetadataState::Current,
    )
    .await
    .unwrap();

    let deal_id = match accepted_by {
        Some(accepted_by) => {
            sqlx::query_scalar!(
                r#"INSERT INTO deals (state, accepted_by, accepted_at) VALUES ($1, $2, DATETIME('now')) RETURNING id;"#,
                deal_state,
                accepted_by
            )
                .fetch_one(database)
                .await
        }
        None => {
            sqlx::query_scalar!(
                r#"INSERT INTO deals (state) VALUES ($1) RETURNING id;"#,
                deal_state
            )
            .fetch_one(database)
            .await
        }
    };

    let deal_id = deal_id.unwrap();

    let segment_id = create_snapshot_segment(database, deal_id.to_string(), size.unwrap_or(262144))
        .await
        .unwrap();
    let snapshot_id = create_snapshot(database, metadata_id, SnapshotState::Pending)
        .await
        .unwrap();
    create_snapshot_segment_association(database, snapshot_id, segment_id)
        .await
        .unwrap();
    Ok(deal_id)
}

pub(crate) async fn create_snapshot_segment_association(
    database: &Database,
    snapshot_id: String,
    segment_id: String,
) -> Result<SqliteQueryResult, sqlx::Error> {
    sqlx::query!(
        r#"INSERT INTO snapshot_segment_associations (snapshot_id, segment_id) VALUES ($1, $2);"#,
        snapshot_id,
        segment_id,
    )
    .execute(database)
    .await
}

pub(crate) async fn create_snapshot_segment(
    database: &Database,
    deal_id: String,
    size: i64,
) -> Result<String, sqlx::Error> {
    sqlx::query_scalar!(
        r#"INSERT INTO snapshot_segments (deal_id, size)
           VALUES ($1, $2)
           RETURNING id;"#,
        deal_id,
        size
    )
    .fetch_one(database)
    .await
}

pub(crate) async fn create_snapshot(
    database: &Database,
    metadata_id: String,
    snapshot_state: SnapshotState,
) -> Result<String, sqlx::Error> {
    let snapshot_state = snapshot_state.to_string();
    sqlx::query_scalar!(
        r#"INSERT INTO snapshots (metadata_id, state)
           VALUES ($1, $2)
           RETURNING id;"#,
        metadata_id,
        snapshot_state
    )
    .fetch_one(database)
    .await
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
    let user_id = sample_user(db).await;

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

pub(crate) async fn sample_user(db: &Database) -> String {
    let uuid = uuid::Uuid::new_v4().to_string();
    create_user(
        db,
        &format!("jessica_{uuid}@sample.users.org"),
        &format!("Jessica {uuid} Tester"),
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
