use sqlx::sqlite::SqlitePoolOptions;
use time::OffsetDateTime;

use crate::database::models::BandwidthMetrics;
use crate::database::{Database, DatabaseConnection};
pub(crate) async fn create_storage_grant(
    conn: &Database,
    client_id: &str,
    grant_id: &str,
    allowed_storage: i64,
) -> String {
    sqlx::query_scalar!(
        r#"INSERT INTO storage_grants (grant_id, client_id, allowed_storage)
                VALUES ($1, $2, $3)
                RETURNING id;"#,
        grant_id,
        client_id,
        allowed_storage,
    )
    .fetch_one(conn)
    .await
    .expect("storage grant creation")
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
