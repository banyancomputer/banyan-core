use std::ops::Range;

use jwt_simple::algorithms::ES384KeyPair;
use rand::Rng;
use sqlx::sqlite::{SqlitePoolOptions, SqliteQueryResult};
use time::OffsetDateTime;
use uuid::Uuid;

use super::models::NewStorageGrant;
use crate::database::models::{BucketType, DealState, MetadataState, SnapshotState, StorageClass};
use crate::database::{Database, DatabaseConnection};
use crate::extractors::{ApiIdentity, ApiIdentityBuilder, SessionIdentity, SessionIdentityBuilder};
use crate::tasks::BLOCK_SIZE;
use crate::utils::keys::fingerprint_public_key;

pub(crate) async fn link_storage_metadata_and_grant(
    conn: &mut DatabaseConnection,
    storage_host_id: &str,
    metadata_id: &str,
    storage_grant_id: &str,
) {
    sqlx::query!(
            "INSERT INTO storage_hosts_metadatas_storage_grants (storage_host_id, metadata_id, storage_grant_id) VALUES ($1, $2, $3);",
            storage_host_id,
            metadata_id,
            storage_grant_id,
        )
            .execute(&mut *conn)
            .await
            .expect("storage_host_metadata");
}

pub(crate) async fn associate_blocks(
    conn: &mut DatabaseConnection,
    metadata_id: &str,
    storage_host_id: &str,
    block_ids: impl Iterator<Item = &str>,
) {
    let now = OffsetDateTime::now_utc();
    for bid in block_ids {
        sqlx::query!(
            "INSERT INTO block_locations (metadata_id, storage_host_id, block_id, stored_at) VALUES ($1, $2, $3, $4);",
            metadata_id,
            storage_host_id,
            bid,
            now
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
) -> String {
    let host_url = host_url.to_string();
    let host_name = host_name.to_string();
    let staging = host_name.contains("staging");
    sqlx::query_scalar!(
        "INSERT INTO storage_hosts (name, url, fingerprint, pem, region, used_storage, available_storage, staging)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING id;
        ",
        host_name,
        host_url,
        "fingerprint_1",
        "pem_1",
        "North America", 
        0,
        0,
        staging
    )
    .fetch_one(database)
    .await
    .expect("storage host creation")
}

pub(crate) async fn create_user_and_associated_data(
    database: &mut DatabaseConnection,
) -> Result<(String, String), sqlx::Error> {
    let user_email = format!("deal_user{}@test.tld", Uuid::new_v4());
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
    Ok((user_id, metadata_id))
}

pub(crate) async fn create_deal(
    database: &mut DatabaseConnection,
    deal_state: DealState,
    size: Option<i64>,
    accepted_by: Option<String>,
) -> Result<String, sqlx::Error> {
    let (_user_id, metadata_id) = create_user_and_associated_data(database).await?;
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
    let size = size.unwrap_or(BLOCK_SIZE);
    let random_number = rand::thread_rng().gen_range(10000..10000000);
    let number_of_blocks = 2;
    let initial_cids: Vec<_> = generate_cids(data_generator(
        random_number..random_number + number_of_blocks,
    ))
    .collect();
    let block_ids = create_blocks(database, initial_cids.iter().map(String::as_str)).await;
    create_snapshot_entries(
        database,
        deal_id.to_string(),
        metadata_id,
        size * number_of_blocks as i64,
        block_ids,
    )
    .await?;

    Ok(deal_id)
}

pub(crate) async fn create_snapshot_entries(
    database: &mut DatabaseConnection,
    deal_id: String,
    metadata_id: String,
    deal_size: i64,
    block_ids: Vec<String>,
) -> Result<(String, String), sqlx::Error> {
    let segment_id = create_snapshot_segment(database, deal_id, deal_size).await?;
    let snapshot_id = create_snapshot(
        database,
        &metadata_id,
        SnapshotState::Pending,
        Some(deal_size),
    )
    .await;
    create_snapshot_segment_association(database, &snapshot_id, &segment_id).await?;
    create_snapshot_block_locations(database, &snapshot_id, block_ids).await;

    Ok((snapshot_id, segment_id))
}
pub(crate) async fn create_snapshot_segment_association(
    database: &mut DatabaseConnection,
    snapshot_id: &str,
    segment_id: &str,
) -> Result<SqliteQueryResult, sqlx::Error> {
    sqlx::query!(
        r#"INSERT INTO snapshot_segment_associations (snapshot_id, segment_id) VALUES ($1, $2);"#,
        snapshot_id,
        segment_id,
    )
    .execute(database)
    .await
}

pub(crate) async fn create_snapshot_block_locations(
    database: &mut DatabaseConnection,
    snapshot_id: &str,
    block_ids: Vec<String>,
) {
    for block_id in block_ids {
        sqlx::query!(
            r#"INSERT INTO snapshot_block_locations (snapshot_id, block_id) VALUES ($1, $2);"#,
            snapshot_id,
            block_id,
        )
        .execute(&mut *database)
        .await
        .expect("snapshot block location creation");
    }
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
    size: Option<i64>,
) -> String {
    let snapshot_state = snapshot_state.to_string();
    let size = size.unwrap_or(BLOCK_SIZE);
    sqlx::query_scalar!(
        r#"INSERT INTO snapshots (metadata_id, state, size, tokens_used)
           VALUES ($1, $2, $3, $3)
           RETURNING id;"#,
        metadata_id,
        snapshot_state,
        size,
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
    let now = OffsetDateTime::now_utc();
    sqlx::query_scalar!(
        r#"INSERT INTO
                buckets (user_id, name, type, storage_class, updated_at)
                VALUES ($1, $2, $3, $4, $5)
                RETURNING id;"#,
        user_id,
        name,
        BucketType::Interactive,
        StorageClass::Hot,
        now
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

pub(crate) async fn create_storage_grant(
    conn: &mut DatabaseConnection,
    storage_host_id: &str,
    user_id: &str,
    authorized_amount: i64,
) -> String {
    let grant = NewStorageGrant {
        storage_host_id,
        user_id,
        authorized_amount,
    }
    .save(conn)
    .await
    .expect("storage grant creation");

    grant.id
}

pub(crate) async fn redeem_storage_grant(
    conn: &mut DatabaseConnection,
    storage_host_id: &str,
    storage_grant_id: &str,
) {
    sqlx::query!(
        r#"
            UPDATE storage_grants
            SET redeemed_at = CURRENT_TIMESTAMP
            WHERE storage_host_id = $1
            AND id = $2
            AND redeemed_at IS NULL;
        "#,
        storage_host_id,
        storage_grant_id
    )
    .execute(conn)
    .await
    .expect("storage grant redemption");
}

pub(crate) async fn associate_upload(
    conn: &mut DatabaseConnection,
    storage_host_id: &str,
    metadata_id: &str,
    storage_grant_id: &str,
) {
    sqlx::query!(
        r#"INSERT INTO storage_hosts_metadatas_storage_grants
               (storage_host_id, metadata_id, storage_grant_id)
               VALUES ($1, $2, $3);"#,
        storage_host_id,
        metadata_id,
        storage_grant_id,
    )
    .execute(&mut *conn)
    .await
    .expect("associate upload");
}

pub(crate) async fn create_storage_host(
    conn: &mut DatabaseConnection,
    name: &str,
    url: &str,
    available_storage: i64,
) -> String {
    let staging = name.contains("staging");
    // Note: this is not creating real fingerprints or public keys but only because the tests
    // haven't needed that level of real data to this point
    sqlx::query_scalar!(
        r#"INSERT INTO storage_hosts (name, url, used_storage, available_storage, region, fingerprint, pem, staging)
               VALUES ($1, $2, 0, $3, 'North America', 'not-a-real-fingerprint', 'not-a-real-pubkey', $4)
               RETURNING id;"#,
        name,
        url,
        available_storage,
        staging
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
    let subscription_id = sqlx::query_scalar!(
        "SELECT id FROM subscriptions WHERE service_key = $1 ORDER BY created_at DESC LIMIT 1;",
        "business",
    )
    .fetch_one(&mut *conn)
    .await
    .expect("business");
    sqlx::query_scalar!(
        r#"INSERT INTO users (email, verified_email, display_name, subscription_id)
                VALUES ($1, true, $2, $3)
                RETURNING id;"#,
        email,
        display_name,
        subscription_id
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
) -> impl Iterator<Item = String> + 'a {
    src_data.map(|d| quick_cid(&d))
}

pub(crate) async fn sample_bucket(conn: &mut DatabaseConnection, user_id: &str) -> String {
    let bucket_name = format!("Bucket-{}", rand::random::<u32>());
    create_hot_bucket(conn, user_id, &bucket_name).await
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
pub(crate) async fn sample_blocks(
    conn: &mut DatabaseConnection,
    number_of_blocks: usize,
    metadata_id: &str,
    storage_host_id: &str,
    grant_id: &str,
) -> Vec<String> {
    let my_uuid = Uuid::new_v4();
    let start = u128::from_le_bytes(*my_uuid.as_bytes()) as usize;
    let initial_cids: Vec<_> =
        generate_cids(data_generator(start..start + number_of_blocks)).collect();
    let block_ids = create_blocks(&mut *conn, initial_cids.iter().map(String::as_str)).await;

    associate_blocks(
        &mut *conn,
        metadata_id,
        storage_host_id,
        block_ids.iter().map(String::as_str),
    )
    .await;

    link_storage_metadata_and_grant(&mut *conn, storage_host_id, metadata_id, grant_id).await;

    block_ids
}

pub(crate) async fn get_or_create_identity(
    conn: &mut DatabaseConnection,
    user_id: &str,
) -> ApiIdentity {
    sqlx::query_scalar!(r#"SELECT email FROM users WHERE id = $1;"#, user_id,)
        .fetch_one(&mut *conn)
        .await
        .expect("user query");

    let device_api_key = sqlx::query!(
        "SELECT id, pem, fingerprint FROM device_api_keys WHERE user_id = $1;",
        user_id
    )
    .fetch_optional(&mut *conn)
    .await
    .expect("device api query");

    match device_api_key {
        Some(api_key) => ApiIdentityBuilder {
            user_id: Uuid::parse_str(user_id).expect("user id"),
            key_fingerprint: api_key.fingerprint.clone(),
        }
        .build(),
        None => {
            let key_pair = ES384KeyPair::generate();
            let pem = key_pair.to_pem().expect("pem key");
            let fingerprint = fingerprint_public_key(&key_pair.public_key());

            sqlx::query_scalar!(
                r#"INSERT INTO device_api_keys (user_id, fingerprint, pem)
                    VALUES ($1, $2, $3)
                    RETURNING id;"#,
                user_id,
                fingerprint,
                pem,
            )
            .fetch_one(&mut *conn)
            .await
            .expect("device api key query");
            ApiIdentityBuilder {
                user_id: Uuid::parse_str(user_id).expect("user id"),
                key_fingerprint: fingerprint.clone(),
            }
            .build()
        }
    }
}
pub(crate) async fn get_or_create_session(
    conn: &mut DatabaseConnection,
    user_id: &str,
) -> SessionIdentity {
    let user_email = sqlx::query_scalar!(r#"SELECT email FROM users WHERE id = $1;"#, user_id,)
        .fetch_one(&mut *conn)
        .await
        .expect("session query");

    let session = sqlx::query!(
        r#"SELECT id, user_id, created_at, expires_at FROM sessions WHERE user_id = $1;"#,
        user_id,
    )
    .fetch_optional(&mut *conn)
    .await
    .expect("session query");

    match session {
        Some(session) => SessionIdentityBuilder {
            session_id: Uuid::parse_str(&session.id).expect("session id"),
            user_id: Uuid::parse_str(&session.user_id).expect("session id"),
            email: user_email,
            created_at: session.created_at,
            expires_at: session.expires_at,
        }
        .build(),
        None => {
            let new_session = sqlx::query!(
                r#"INSERT INTO sessions (user_id, provider, access_token, created_at, expires_at)
                    VALUES ($1, 'google.com', 'access_token', DATETIME('now'), DATETIME('now', '+1 day'))
                    RETURNING id, user_id, created_at, expires_at;"#,
                user_id,
            )
                .fetch_one(&mut *conn)
            .await.expect("session creation");
            SessionIdentityBuilder {
                session_id: Uuid::parse_str(&new_session.id).expect("session id"),
                user_id: Uuid::parse_str(&new_session.user_id).expect("user id"),
                email: user_email,
                created_at: new_session.created_at,
                expires_at: new_session.expires_at,
            }
            .build()
        }
    }
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

pub(crate) fn quick_cid(data: &[u8]) -> String {
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    use base64::Engine;

    let mut cid_bytes = Vec::with_capacity(36);

    cid_bytes.extend_from_slice(&[0x01, 0x55, 0x1e, 0x20]);
    cid_bytes.extend_from_slice(blake3::hash(data).as_bytes());

    let encoded = URL_SAFE_NO_PAD.encode(cid_bytes);

    format!("u{}", encoded)
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
