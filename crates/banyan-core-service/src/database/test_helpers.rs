use std::ops::Range;

use sqlx::sqlite::{SqlitePoolOptions, SqliteQueryResult};
use time::OffsetDateTime;

use crate::database::models::{BucketType, DealState, MetadataState, SnapshotState, StorageClass};
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
    cid_list: impl Iterator<Item = &str>,
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

pub(crate) async fn create_storage_hosts(
    database: &mut DatabaseConnection,
    host_url: &str,
    host_name: &str,
) -> Result<String, sqlx::Error> {
    let host_url = host_url.to_string();
    let host_name = host_name.to_string();
    sqlx::query_scalar!(
            r#"INSERT INTO storage_hosts (id, name, url, fingerprint, pem, region, used_storage, available_storage)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING id;"#,
            host_name,
            host_url,
            "fingerprint_1",
            "pem_1",
            "North America", 
            "hello.com",
            0,
            0
        )
    .fetch_one(database)
    .await
}

pub(crate) async fn create_deal(
    database: &mut DatabaseConnection,
    deal_state: DealState,
    size: Option<i64>,
    accepted_by: Option<String>,
) -> Result<String, sqlx::Error> {
    let user_email = format!("deal_user{}@test.tld", uuid::Uuid::new_v4());
    let user_id = sample_user(database, &user_email).await;
    let bucket_id = create_hot_bucket(database, user_id.as_str(), "test_bucket").await;
    let metadata_id = create_metadata(
        database,
        bucket_id.as_str(),
        format!("root-cid-{}", bucket_id).as_str(),
        format!("metadata-cid-{}", bucket_id).as_str(),
        MetadataState::Current,
        None,
        None,
    )
    .await;

    let deal_id = match accepted_by {
        Some(accepted_by) => {
            sqlx::query_scalar!(
                r#"INSERT INTO deals (state, accepted_by, accepted_at) VALUES ($1, $2, DATETIME('now')) RETURNING id;"#,
                deal_state,
                accepted_by
            )
                .fetch_one(&mut *database)
                .await
        }
        None => {
            sqlx::query_scalar!(
                r#"INSERT INTO deals (state) VALUES ($1) RETURNING id;"#,
                deal_state
            )
            .fetch_one(&mut *database)
            .await
        }
    };

    let deal_id = deal_id.unwrap();

    let segment_id = create_snapshot_segment(database, deal_id.to_string(), size.unwrap_or(262144))
        .await
        .unwrap();
    let snapshot_id = create_snapshot(database, &metadata_id, SnapshotState::Pending).await;
    create_snapshot_segment_association(database, snapshot_id, segment_id)
        .await
        .unwrap();

    Ok(deal_id)
}

pub(crate) async fn create_snapshot_segment_association(
    database: &mut DatabaseConnection,
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
    database: &mut DatabaseConnection,
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
    database: &mut DatabaseConnection,
    metadata_id: &str,
    snapshot_state: SnapshotState,
) -> String {
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
    .expect("snapshot creation")
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
    previous_metadata_cid: Option<String>,
) -> String {
    // Note: be sure to use explicit timestamps to yeild the same precission
    //  we expect as a result from calls to NewMetadata::save(), Metadata::mark_current(),
    //   and Metadata::mark_upload_complete()
    let now = match timestamp {
        Some(ts) => ts,
        None => OffsetDateTime::now_utc(),
    };
    sqlx::query_scalar!(
        r#"INSERT INTO
                metadata (bucket_id, root_cid, metadata_cid, expected_data_size, state, created_at, updated_at, previous_metadata_cid)
                VALUES ($1, $2, $3, 0, $4, $5, $5, $6)
                RETURNING id;"#,
        bucket_id,
        root_cid,
        metadata_cid,
        state,
        now,
        previous_metadata_cid
    )
    .fetch_one(conn)
    .await
    .expect("metadata creation")
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
        r#"INSERT INTO storage_hosts (name, url, used_storage, available_storage, region, fingerprint, pem)
               VALUES ($1, $2, 0, $3, 'North America', 'not-a-real-fingerprint', 'not-a-real-pubkey')
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
        r#"INSERT INTO users (email, verified_email, display_name)
                VALUES ($1, true, $2)
                RETURNING id;"#,
        email,
        display_name,
    )
    .fetch_one(conn)
    .await
    .expect("user creation")
}

pub(crate) fn data_generator<'a>(range: Range<usize>) -> impl Iterator<Item = Vec<u8>> + 'a {
    range.map(|n| n.to_le_bytes().to_vec())
}

pub(crate) fn generate_cids<'a>(
    src_data: impl Iterator<Item = Vec<u8>> + 'a,
) -> impl Iterator<Item = cid::Cid> + 'a {
    use cid::multihash::MultihashDigest;
    src_data.map(|d| cid::Cid::new_v1(0x55, cid::multihash::Code::Blake3_256.digest(d.as_slice())))
}

pub(crate) fn normalize_cids<'a>(
    src_data: impl Iterator<Item = cid::Cid> + 'a,
) -> impl Iterator<Item = String> + 'a {
    src_data.map(|cid| {
        cid.to_string_of_base(cid::multibase::Base::Base64Url)
            .expect("valid conversion")
    })
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

pub(crate) async fn setup_database() -> Database {
    use crate::pricing;

    let pool = SqlitePoolOptions::new()
        .connect("sqlite::memory:")
        .await
        .expect("Failed to connect to the database");

    let mut conn = pool.begin().await.expect("db conn");

    sqlx::migrate!()
        .run(&mut conn)
        .await
        .expect("failed to run migrations");

    pricing::sync_pricing_config(&mut conn, pricing::builtin_pricing_config())
        .await
        .expect("price sync");

    conn.commit().await.expect("db close");

    pool
}

pub(crate) async fn sample_metadata(
    conn: &mut DatabaseConnection,
    bucket_id: &str,
    counter: usize,
    state: MetadataState,
) -> String {
    let root_cid = format!("root-cid-{}", counter);
    let metadata_cid = format!("metadata-cid-{}", counter);

    create_metadata(conn, bucket_id, &metadata_cid, &root_cid, state, None, None).await
}

pub(crate) async fn sample_user(conn: &mut DatabaseConnection, email: &str) -> String {
    create_user(conn, email, "Generic Tester").await
}

pub(crate) async fn metadata_timestamps(
    conn: &mut DatabaseConnection,
    metadata_id: &str,
) -> (OffsetDateTime, OffsetDateTime) {
    let rec = sqlx::query!(
        r#"SELECT 
            created_at as 'created_at: OffsetDateTime',
            updated_at as 'updated_at: OffsetDateTime'
        FROM metadata WHERE id = $1;"#,
        metadata_id,
    )
    .fetch_one(conn)
    .await
    .expect("query success");

    (rec.created_at, rec.updated_at)
}

pub(crate) async fn raw_metadata_timestamps(
    conn: &mut DatabaseConnection,
    metadata_id: &str,
) -> (String, String) {
    let rec = sqlx::query!(
        r#"SELECT 
            created_at as 'created_at: String',
            updated_at as 'updated_at: String'
        FROM metadata WHERE id = $1;"#,
        metadata_id,
    )
    .fetch_one(conn)
    .await
    .expect("query success");

    (rec.created_at, rec.updated_at)
}
