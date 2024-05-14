use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};
use sqlx::encode::IsNull;
use sqlx::error::BoxDynError;
use sqlx::sqlite::{SqliteArgumentValue, SqliteTypeInfo, SqliteValueRef};
use sqlx::{Decode, Encode, QueryBuilder, Sqlite, Type};

use super::UserKey;
use crate::api::models::ApiPushKey;
use crate::database::DatabaseConnection;

#[derive(sqlx::FromRow, Serialize)]
pub struct BucketAccess {
    pub user_key_id: String,
    pub bucket_id: String,
    pub approved: bool,
}

impl BucketAccess {
    pub async fn by_fingerprint(
        conn: &mut DatabaseConnection,
        fingerprint: &str,
    ) -> Result<BucketAccess, sqlx::Error> {
        sqlx::query_as!(
            BucketAccess,
            r#"
                SELECT ba.* FROM bucket_access AS ba
                JOIN user_keys AS uk ON uk.id = ba.user_key_id
                WHERE uk.fingerprint = $1;
            "#,
            fingerprint,
        )
        .fetch_one(&mut *conn)
        .await
    }

    /// Performed on the Push of metadata
    pub async fn update_access_associations(
        conn: &mut DatabaseConnection,
        bucket_id: &str,
        user_id: &str,
        user_keys: &[ApiPushKey],
    ) -> Result<(), sqlx::Error> {
        // Fingerprints of the keys pushed
        let provided_prints: Vec<String> = user_keys
            .iter()
            .map(|k| k.fingerprint.to_string())
            .collect();
        // List all keys associated with the bucket
        let existing_prints: Vec<String> = sqlx::query_scalar!(
            r#"
                SELECT fingerprint FROM user_keys
                JOIN bucket_access AS ba ON ba.user_key_id = id
                WHERE ba.bucket_id = $1
            "#,
            bucket_id
        )
        .fetch_all(&mut *conn)
        .await?;

        let new_keys: Vec<ApiPushKey> = user_keys
            .iter()
            .filter(|&k| !existing_prints.contains(&k.fingerprint))
            .cloned()
            .collect();

        // Any not present in the request should be revoked
        let (approve_prints, revoke_prints): (Vec<_>, Vec<_>) = existing_prints
            .into_iter()
            .partition(|k| provided_prints.contains(k));

        // Revoke unused keys
        // Self::set_group(conn, bucket_id, &revoke_prints, false).await?;

        // Create new keys
        for key in &new_keys {
            if UserKey::by_fingerprint(conn, &key.fingerprint)
                .await
                .is_err()
            {
                UserKey::create(
                    conn,
                    "unknown",
                    user_id,
                    &key.fingerprint,
                    &key.public_key,
                    false,
                )
                .await?;
            }
        }
        let new_key_prints: Vec<String> = new_keys.into_iter().map(|k| k.fingerprint).collect();

        // Approve all keys being utilized
        Self::set_group(
            conn,
            bucket_id,
            &[approve_prints, new_key_prints].concat(),
            true,
        )
        .await?;

        Ok(())
    }

    pub async fn set_group(
        conn: &mut DatabaseConnection,
        bucket_id: &str,
        user_key_fingerprints: &[String],
        approved: bool,
    ) -> Result<(), sqlx::Error> {
        let mut builder = QueryBuilder::new(
            r#"
                INSERT OR REPLACE INTO bucket_access (user_key_id, bucket_id, state)
                SELECT id,
            "#,
        );
        builder.push_bind(bucket_id);
        builder.push(r#" AS bucket_id, "#);
        builder.push_bind(approved);
        builder.push(
            r#" AS approved
                FROM user_keys AS uk
                WHERE uk.fingerprint IN (
            "#,
        );
        let mut separator = builder.separated(", ");
        for user_key_fingerprint in user_key_fingerprints {
            separator.push_bind(user_key_fingerprint);
        }
        builder.push(r#");"#);
        builder.build().execute(&mut *conn).await?;
        Ok(())
    }

    pub async fn set(
        conn: &mut DatabaseConnection,
        bucket_id: &str,
        user_key_fingerprint: &str,
        approved: bool,
    ) -> Result<BucketAccess, sqlx::Error> {
        let access = sqlx::query_as!(
            BucketAccess,
            r#"
                INSERT OR REPLACE INTO bucket_access (user_key_id, bucket_id, approved)
                SELECT uk.id AS user_key_id, $1 AS bucket_id, $2 AS approved
                FROM user_keys AS uk
                WHERE uk.fingerprint = $3
                RETURNING user_key_id, bucket_id, approved;
            "#,
            bucket_id,
            approved,
            user_key_fingerprint,
        )
        .fetch_one(&mut *conn)
        .await?;

        Ok(access)
    }
}
