use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::{
    database::{
        models::{BucketAccess, BucketAccessState},
        DatabaseConnection,
    },
    extractors::{ApiIdentity, UserIdentity},
};

#[derive(sqlx::FromRow, Serialize)]
pub struct UserKey {
    pub id: String,
    pub name: String,
    pub user_id: String,

    pub api_access: bool,

    pub pem: String,
    pub fingerprint: String,

    //
    pub updated_at: OffsetDateTime,
    pub created_at: OffsetDateTime,
}

impl UserKey {
    pub async fn from_fingerprint(
        conn: &mut DatabaseConnection,
        fingerprint: &str,
    ) -> Result<Self, sqlx::Error> {
        sqlx::query_as!(
            UserKey,
            r#"
                SELECT * FROM user_keys
                WHERE fingerprint = $2;
            "#,
            fingerprint,
        )
        .fetch_one(&mut *conn)
        .await
    }

    pub async fn can_access_bucket(
        conn: &mut DatabaseConnection,
        user_id: &str,
        bucket_id: &str,
    ) -> Result<bool, sqlx::Error> {
        sqlx::query_as!(
            BucketAccess,
            r#"
                SELECT ba.user_key_id, ba.bucket_id, ba.state AS 'state: BucketAccessState' FROM bucket_access AS ba
                JOIN buckets AS b ON b.id = ba.bucket_id
                JOIN user_keys AS uk ON uk.id = ba.user_key_id
                JOIN users AS u ON u.id = uk.user_id
                WHERE u.id = $1
                AND b.id = $2;
            "#,
            user_id,
            bucket_id,
        )
        .fetch_optional(&mut *conn)
        .await
        .map(|v| v.is_some())
    }
}
