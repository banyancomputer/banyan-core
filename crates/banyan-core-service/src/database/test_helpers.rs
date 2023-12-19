use sqlx::sqlite::SqlitePoolOptions;
use time::OffsetDateTime;

use crate::database::models::{BucketType, MetadataState, StorageClass};
use crate::database::{Database, DatabaseConnection};

pub(crate) async fn associate_blocks(
    conn: &mut DatabaseConnection,
    metadata_id: &str,
    storage_host_id: &str,
    block_ids: impl Iterator<Item = &str>,
) {
    for bid in block_ids {
        sqlx::query!(
            "INSERT INTO block_locations (metadata_id, storage_host_id, block_id) VALUES ($1, $2, $3);",
            metadata_id,
            storage_host_id,
            bid,
        )
        .execute(&mut *conn)
        .await
        .expect("blocks to associate");
    }
}

pub(crate) async fn assert_metadata_in_state(
    conn: &mut DatabaseConnection,
    metadata_id: &str,
    expected_state: MetadataState,
) {
    let db_state = sqlx::query_scalar!(
        r#"SELECT state as 'state: MetadataState' FROM metadata WHERE id = $1;"#,
        metadata_id,
    )
    .fetch_one(&mut *conn)
    .await
    .expect("query success");

    assert_eq!(
        db_state, expected_state,
        "metadata was not in expected state"
    );
}

pub(crate) async fn create_blocks(
    conn: &mut DatabaseConnection,
    cid_list: impl Iterator<Item = cid::Cid>,
) -> Vec<String> {
    let mut block_ids = Vec::new();

    for cid in cid_list {
        let bid = sqlx::query_scalar!("INSERT INTO blocks (cid) VALUES ($1) RETURNING id;", cid)
            .fetch_one(&mut *conn)
            .await
            .expect("block creation");

        block_ids.push(bid);
    }

    block_ids
}

pub(crate) async fn create_bucket_key(
    conn: &mut DatabaseConnection,
    bucket_id: &str,
    public_key: &str,
    fingerprint: &str,
    approved: bool,
) -> String {
    sqlx::query_scalar!(
        r#"INSERT INTO bucket_keys (bucket_id, pem, fingerprint, approved)
                VALUES ($1, $2, $3, $4)
                RETURNING id;"#,
        bucket_id,
        public_key,
        fingerprint,
        approved,
    )
    .fetch_one(&mut *conn)
    .await
    .expect("bucket key creation")
}

pub(crate) async fn create_hot_bucket(
    conn: &mut DatabaseConnection,
    user_id: &str,
    name: &str,
) -> String {
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
    .fetch_one(conn)
    .await
    .expect("hot bucket creation")
}

pub(crate) async fn create_metadata(
    conn: &mut DatabaseConnection,
    bucket_id: &str,
    metadata_cid: &str,
    root_cid: &str,
    state: MetadataState,
    timestamp: Option<OffsetDateTime>,
) -> String {
    if let Some(ts) = timestamp {
        sqlx::query_scalar!(
            r#"INSERT INTO
                    metadata (bucket_id, root_cid, metadata_cid, expected_data_size, state, created_at, updated_at)
                    VALUES ($1, $2, $3, 0, $4, $5, $5)
                    RETURNING id;"#,
            bucket_id,
            root_cid,
            metadata_cid,
            state,
            ts,
        )
        .fetch_one(conn)
        .await
        .expect("metadata creation")
    } else {
        sqlx::query_scalar!(
            r#"INSERT INTO
                    metadata (bucket_id, root_cid, metadata_cid, expected_data_size, state)
                    VALUES ($1, $2, $3, 0, $4)
                    RETURNING id;"#,
            bucket_id,
            root_cid,
            metadata_cid,
            state,
        )
        .fetch_one(conn)
        .await
        .expect("metadata creation")
    }
}

pub(crate) async fn create_storage_host(
    conn: &mut DatabaseConnection,
    name: &str,
    url: &str,
    available_storage: i64,
) -> String {
    // Note: this is not creating real fingerprints or public keys but only because the tests
    // haven't needed that level of real data to this point
    sqlx::query_scalar!(
        r#"INSERT INTO storage_hosts (name, url, used_storage, available_storage, fingerprint, pem)
               VALUES ($1, $2, 0, $3, 'not-a-real-fingerprint', 'not-a-real-pubkey')
               RETURNING id;"#,
        name,
        url,
        available_storage,
    )
    .fetch_one(&mut *conn)
    .await
    .expect("creation of storage host")
}

pub(crate) async fn create_user(
    conn: &mut DatabaseConnection,
    email: &str,
    display_name: &str,
) -> String {
    sqlx::query_scalar!(
        r#"INSERT INTO
                users (email, verified_email, display_name)
                VALUES ($1, true, $2)
                RETURNING id;"#,
        email,
        display_name,
    )
    .fetch_one(conn)
    .await
    .expect("user creation")
}

pub(crate) async fn pending_metadata(
    conn: &mut DatabaseConnection,
    bucket_id: &str,
    counter: usize,
) -> String {
    sample_metadata(conn, bucket_id, counter, MetadataState::Pending).await
}

pub(crate) async fn sample_bucket(conn: &mut DatabaseConnection, user_id: &str) -> String {
    create_hot_bucket(conn, user_id, "Habernero").await
}

pub(crate) async fn sample_metadata(
    conn: &mut DatabaseConnection,
    bucket_id: &str,
    counter: usize,
    state: MetadataState,
) -> String {
    let root_cid = format!("root-cid-{}", counter);
    let metadata_cid = format!("metadata-cid-{}", counter);

    create_metadata(conn, bucket_id, &metadata_cid, &root_cid, state, None).await
}

pub(crate) async fn sample_user(conn: &mut DatabaseConnection, email: &str) -> String {
    create_user(conn, email, "Generic Tester").await
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
