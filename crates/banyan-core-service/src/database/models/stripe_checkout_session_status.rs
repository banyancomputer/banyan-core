use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};
use sqlx::encode::IsNull;
use sqlx::error::BoxDynError;
use sqlx::sqlite::{SqliteArgumentValue, SqliteTypeInfo, SqliteValueRef};
use sqlx::{Decode, Encode, Sqlite, Type};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StripeCheckoutSessionStatus {
    Created,
    Completed,
    Expired,
}

impl Display for StripeCheckoutSessionStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            StripeCheckoutSessionStatus::Created => f.write_str("created"),
            StripeCheckoutSessionStatus::Completed => f.write_str("completed"),
            StripeCheckoutSessionStatus::Expired => f.write_str("expired"),
        }
    }
}

impl TryFrom<&str> for StripeCheckoutSessionStatus {
    type Error = StripeCheckoutSessionStatusError;

    fn try_from(val: &str) -> Result<Self, StripeCheckoutSessionStatusError> {
        let variant = match val {
            "completed" => StripeCheckoutSessionStatus::Completed,
            "created" => StripeCheckoutSessionStatus::Created,
            "expired" => StripeCheckoutSessionStatus::Expired,
            _ => return Err(StripeCheckoutSessionStatusError::InvalidValue),
        };

        Ok(variant)
    }
}

impl Decode<'_, Sqlite> for StripeCheckoutSessionStatus {
    fn decode(value: SqliteValueRef<'_>) -> Result<Self, BoxDynError> {
        let inner_val = <&str as Decode<Sqlite>>::decode(value)?;
        Self::try_from(inner_val).map_err(Into::into)
    }
}

impl Encode<'_, Sqlite> for StripeCheckoutSessionStatus {
    fn encode_by_ref(&self, args: &mut Vec<SqliteArgumentValue<'_>>) -> IsNull {
        args.push(SqliteArgumentValue::Text(self.to_string().into()));
        IsNull::No
    }
}

impl Type<Sqlite> for StripeCheckoutSessionStatus {
    fn compatible(ty: &SqliteTypeInfo) -> bool {
        <&str as Type<Sqlite>>::compatible(ty)
    }

    fn type_info() -> SqliteTypeInfo {
        <&str as Type<Sqlite>>::type_info()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum StripeCheckoutSessionStatusError {
    #[error("attempted to decode unknown status value")]
    InvalidValue,
}
