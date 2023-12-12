use serde::Serialize;
use time::OffsetDateTime;

use crate::database::DatabaseConnection;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct User {
    pub id: String,
    pub email: String,
    pub verified_email: bool,
    pub display_name: String,
    pub locale: Option<String>,
    pub profile_image: Option<String>,
    pub created_at: OffsetDateTime,
    pub accepted_tos_at: Option<OffsetDateTime>,
}

impl User {
    pub async fn consumed_storage(
        conn: &mut DatabaseConnection,
        user_id: &str,
    ) -> Result<i64, sqlx::Error> {
        sqlx::query_scalar!(
            r#"SELECT
                COALESCE(SUM(m.metadata_size), 0) + COALESCE(SUM(COALESCE(m.data_size, m.expected_data_size)), 0) AS '!size'
            FROM
                metadata m
            INNER JOIN
                buckets b ON b.id = m.bucket_id
            WHERE
                b.user_id = $1 AND m.state IN ('current', 'outdated', 'pending');"#,
            user_id,
        )
        .fetch(&mut *conn)
        .await
    }
}
