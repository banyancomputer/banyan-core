use std::fmt::{self, Display, Formatter};

use crate::database::models::UserKey;
use crate::database::DatabaseConnection;
use serde::{Deserialize, Serialize};
use sqlx::encode::IsNull;
use sqlx::error::BoxDynError;
use sqlx::sqlite::{SqliteArgumentValue, SqliteTypeInfo, SqliteValueRef};
use sqlx::{Decode, Encode, Sqlite, Type};

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
        tracing::warn!("looking key up by fingerprint {fingerprint}");
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
}

impl Display for BucketAccessState {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            BucketAccessState::Pending => f.write_str("pending"),
            BucketAccessState::Approved => f.write_str("pending"),
            BucketAccessState::Revoked => f.write_str("pending"),
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
