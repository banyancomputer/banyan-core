use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};
use sqlx::encode::IsNull;
use sqlx::error::BoxDynError;
use sqlx::sqlite::{SqliteArgumentValue, SqliteTypeInfo, SqliteValueRef};
use sqlx::{Decode, Encode, QueryBuilder, Sqlite, Type};

use crate::database::DatabaseConnection;

#[derive(sqlx::FromRow, Serialize)]
pub struct BucketAccess {
    pub user_key_id: String,
    pub bucket_id: String,
    pub state: BucketAccessState,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BucketAccessState {
    Pending,
    Approved,
    Revoked,
}

impl BucketAccess {
    pub async fn by_fingerprint(
        conn: &mut DatabaseConnection,
        fingerprint: &str,
    ) -> Result<BucketAccess, sqlx::Error> {
        sqlx::query_as!(
            BucketAccess,
            r#"
                SELECT ba.user_key_id, ba.bucket_id, ba.state AS 'state: BucketAccessState' FROM bucket_access AS ba
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
        user_key_ids: &[String],
    ) -> Result<(), sqlx::Error> {
        // List all keys associated with the bucket
        let existing_key_ids: Vec<String> = sqlx::query_scalar!(
            r#"
                SELECT id FROM user_keys
                JOIN bucket_access AS ba ON ba.user_key_id = id
                WHERE ba.bucket_id = $1
            "#,
            bucket_id
        )
        .fetch_all(&mut *conn)
        .await?;

        // Any not present in the request should be revoked
        let revoked_keys: Vec<String> = existing_key_ids
            .into_iter()
            .filter(|id| !user_key_ids.contains(id))
            .collect();

        // Revoke keys appropriately
        Self::set_group(conn, bucket_id, &revoked_keys, BucketAccessState::Revoked).await?;

        Self::set_group(conn, bucket_id, &user_key_ids, BucketAccessState::Approved).await
    }

    pub async fn set_group(
        conn: &mut DatabaseConnection,
        bucket_id: &str,
        user_key_ids: &[String],
        state: BucketAccessState,
    ) -> Result<(), sqlx::Error> {
        let mut builder = QueryBuilder::new(
            r#"
                INSERT OR REPLACE INTO bucket_access (user_key_id, bucket_id, state)
                SELECT id,
            "#,
        );
        builder.push_bind(bucket_id);
        builder.push(r#" AS bucket_id, "#);
        builder.push_bind(state);
        builder.push(
            r#" AS state
                FROM user_keys AS uk
                WHERE uk.id IN (
            "#,
        );
        let mut separator = builder.separated(", ");
        for user_key_id in user_key_ids {
            separator.push_bind(user_key_id);
        }
        builder.push(r#");"#);
        builder.build().execute(&mut *conn).await?;
        Ok(())
    }

    pub async fn set(
        conn: &mut DatabaseConnection,
        user_key_id: &str,
        bucket_id: &str,
        state: BucketAccessState,
    ) -> Result<BucketAccess, sqlx::Error> {
        let access = sqlx::query_as!(
            BucketAccess,
            r#"
                INSERT OR REPLACE INTO bucket_access (user_key_id, bucket_id, state)
                VALUES ($1, $2, $3)
                RETURNING user_key_id, bucket_id, state as 'state: BucketAccessState';
            "#,
            user_key_id,
            bucket_id,
            state
        )
        .fetch_one(&mut *conn)
        .await?;

        Ok(access)
    }
}

impl Display for BucketAccessState {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            BucketAccessState::Pending => f.write_str("pending"),
            BucketAccessState::Approved => f.write_str("approved"),
            BucketAccessState::Revoked => f.write_str("revoked"),
        }
    }
}

impl TryFrom<&str> for BucketAccessState {
    type Error = BucketAccessStateError;

    fn try_from(val: &str) -> Result<Self, BucketAccessStateError> {
        let variant = match val {
            "pending" => BucketAccessState::Pending,
            "approved" => BucketAccessState::Approved,
            "revoked" => BucketAccessState::Revoked,
            _ => return Err(BucketAccessStateError::InvalidStateValue),
        };

        Ok(variant)
    }
}

impl Decode<'_, Sqlite> for BucketAccessState {
    fn decode(value: SqliteValueRef<'_>) -> Result<Self, BoxDynError> {
        let inner_val = <&str as Decode<Sqlite>>::decode(value)?;
        Self::try_from(inner_val).map_err(Into::into)
    }
}

impl Encode<'_, Sqlite> for BucketAccessState {
    fn encode_by_ref(&self, args: &mut Vec<SqliteArgumentValue<'_>>) -> IsNull {
        args.push(SqliteArgumentValue::Text(self.to_string().into()));
        IsNull::No
    }
}

impl Type<Sqlite> for BucketAccessState {
    fn compatible(ty: &SqliteTypeInfo) -> bool {
        <&str as Type<Sqlite>>::compatible(ty)
    }

    fn type_info() -> SqliteTypeInfo {
        <&str as Type<Sqlite>>::type_info()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum BucketAccessStateError {
    #[error("attempted to decode unknown state value")]
    InvalidStateValue,
}
