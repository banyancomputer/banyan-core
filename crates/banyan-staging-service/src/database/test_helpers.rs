use std::ops::Range;

use banyan_object_store::{ObjectStore, ObjectStoreConnection, ObjectStorePath};
use bytes::Bytes;
use sqlx::sqlite::SqlitePoolOptions;
use time::OffsetDateTime;

use crate::database::models::BandwidthMetrics;
use crate::database::Database;

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

pub(crate) async fn create_storage_grant(
    db: &Database,
    client_id: &str,
    grant_id: &str,
    allowed_storage: i64,
) -> String {
    sqlx::query_scalar!(
        r#"INSERT INTO storage_grants (client_id, grant_id, allowed_storage)
                VALUES ($1, $2, $3)
                RETURNING id;"#,
        client_id,
        grant_id,
        allowed_storage,
    )
    .fetch_one(db)
    .await
    .expect("storage grant creation")
}

pub(crate) async fn create_upload(
    db: &Database,
    client_id: &str,
    metadata_id: &str,
    reported_size: i64,
) -> String {
    sqlx::query_scalar!(
        r#"INSERT INTO uploads (client_id, metadata_id, reported_size, base_path, state)
                VALUES ($1, $2, $3, $4, $5)
                RETURNING id;"#,
        client_id,
        metadata_id,
        reported_size,
        "file:://tmp",
        "complete",
    )
    .fetch_one(db)
    .await
    .expect("upload creation")
}

pub(crate) async fn create_blocks(
    db: &Database,
    cid_list: impl Iterator<Item = &str>,
) -> Vec<String> {
    let mut block_ids = Vec::new();

    for cid in cid_list {
        let data_length = rand::random::<u8>() as i64 % 100 + 1;
        let bid = sqlx::query_scalar!(
            "INSERT INTO blocks (cid, data_length) VALUES ($1, $2) RETURNING id;",
            cid,
            data_length
        )
        .fetch_one(db)
        .await
        .expect("block creation");

        block_ids.push(bid);
    }

    block_ids
}

pub(crate) async fn associate_blocks_to_upload(
    db: &Database,
    upload_id: &str,
    block_cids: Vec<String>,
) {
    for bid in block_cids.iter() {
        sqlx::query_scalar!(
            "INSERT INTO uploads_blocks (upload_id, block_id) VALUES ($1, $2);",
            upload_id,
            bid,
        )
        .execute(db)
        .await
        .expect("block creation");
    }
}
pub(crate) async fn sample_blocks(
    db: &Database,
    number_of_blocks: usize,
    upload_id: &str,
) -> Vec<String> {
    let initial_cids: Vec<_> =
        normalize_cids(generate_cids(data_generator(0..number_of_blocks))).collect();
    let block_ids = create_blocks(&db, initial_cids.iter().map(String::as_str)).await;
    associate_blocks_to_upload(&db, &upload_id, block_ids.clone()).await;

    block_ids
}
pub(crate) async fn save_blocks_to_storage(
    store_connection: &ObjectStoreConnection,
    metadata_id: &str,
    block_cids: Vec<String>,
) {
    let store = ObjectStore::new(&store_connection).expect("store creation");
    for cid in block_cids {
        let location = ObjectStorePath::from(format!("{}/{}.bin", &metadata_id, cid));
        store
            .put(&location, Bytes::from(cid.to_string().into_bytes()))
            .await
            .expect("block storage");
    }
}

pub(crate) async fn create_client(
    conn: &Database,
    platform_id: &str,
    fingerprint: &str,
    public_key: &str,
) -> String {
    sqlx::query_scalar!(
        r#"INSERT INTO clients (platform_id, fingerprint, public_key)
                VALUES ($1, $2, $3)
                RETURNING id;"#,
        platform_id,
        fingerprint,
        public_key,
    )
    .fetch_one(conn)
    .await
    .expect("client creation")
}

pub(crate) async fn create_bandwidth_metric(
    conn: &Database,
    user_id: &str,
    ingress: i64,
    egress: i64,
    created_at: OffsetDateTime,
) {
    BandwidthMetrics {
        user_id: user_id.to_string(),
        ingress,
        egress,
        created_at,
    }
    .save(conn)
    .await
    .unwrap()
}

pub(crate) async fn setup_database() -> Database {
    let pool = SqlitePoolOptions::new()
        .connect("sqlite::memory:")
        .await
        .expect("Failed to connect to the database");

    let mut conn = pool.begin().await.expect("db conn");

    sqlx::migrate!()
        .run(&mut conn)
        .await
        .expect("failed to run migrations");

    conn.commit().await.expect("db close");

    pool
}
