use time::OffsetDateTime;

use crate::database::DatabaseConnection;

#[derive(sqlx::FromRow)]
pub struct Subscription {
    id: String,

    price_key: String,
    title: String,

    stripe_product_id: Option<String>,

    allow_overage: bool,
    archival_available: bool,
    visible: bool,

    base_price: Option<i64>,
    storage_overage_price: Option<i64>,
    bandwidth_overage_price: Option<i64>,

    included_archival: i64,
    included_bandwidth: i64,
    included_storage: i64,

    archival_hard_limit: Option<i64>,
    bandwidth_hard_limit: Option<i64>,
    storage_hard_limit: Option<i64>,

    created_at: OffsetDateTime,
}

impl Subscription {
    pub async fn active_price_key(conn: &mut DatabaseConnection, price_key: &str) -> Result<Option<Subscription>, sqlx::Error> {
        sqlx::query_as!(
            Self,
            r#"SELECT * FROM subscriptions
                   WHERE price_key = $1
                   ORDER BY created_at DESC
                   LIMIT 1;"#,
            price_key,
        )
        .fetch_optional(&mut *conn)
        .await
    }
}
