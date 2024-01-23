use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};
use sqlx::encode::IsNull;
use sqlx::error::BoxDynError;
use sqlx::sqlite::{SqliteArgumentValue, SqliteTypeInfo, SqliteValueRef};
use sqlx::{Decode, Encode, Sqlite, Type};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SubscriptionStatus {
    Active,
    Canceled,
    Incomplete,
    IncompleteExpired,
    PastDue,
    Paused,
    Trialing,
    Unpaid,
}

impl Display for SubscriptionStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            SubscriptionStatus::Active => f.write_str("active"),
            SubscriptionStatus::Canceled => f.write_str("canceled"),
            SubscriptionStatus::IncompleteExpired => f.write_str("incomplete_expired"),
            SubscriptionStatus::Incomplete => f.write_str("incomplete"),
            SubscriptionStatus::PastDue => f.write_str("past_due"),
            SubscriptionStatus::Paused => f.write_str("paused"),
            SubscriptionStatus::Trialing => f.write_str("trialing"),
            SubscriptionStatus::Unpaid => f.write_str("unpaid"),
        }
    }
}

use stripe::generated::billing::subscription::SubscriptionStatus as StripeSubscriptionStatus;

impl From<StripeSubscriptionStatus> for SubscriptionStatus {
    fn from(val: StripeSubscriptionStatus) -> Self {
        match val {
            StripeSubscriptionStatus::Active => SubscriptionStatus::Active,
            StripeSubscriptionStatus::Canceled => SubscriptionStatus::Canceled,
            StripeSubscriptionStatus::IncompleteExpired => SubscriptionStatus::IncompleteExpired,
            StripeSubscriptionStatus::Incomplete => SubscriptionStatus::Incomplete,
            StripeSubscriptionStatus::PastDue => SubscriptionStatus::PastDue,
            StripeSubscriptionStatus::Paused => SubscriptionStatus::Paused,
            StripeSubscriptionStatus::Trialing => SubscriptionStatus::Trialing,
            StripeSubscriptionStatus::Unpaid => SubscriptionStatus::Unpaid,
        }
    }
}

impl TryFrom<&str> for SubscriptionStatus {
    type Error = SubscriptionStatusError;

    fn try_from(val: &str) -> Result<Self, SubscriptionStatusError> {
        let variant = match val {
            "active" => SubscriptionStatus::Active,
            "canceled" => SubscriptionStatus::Canceled,
            "incomplete_expired" => SubscriptionStatus::IncompleteExpired,
            "incomplete" => SubscriptionStatus::Incomplete,
            "past_due" => SubscriptionStatus::PastDue,
            "paused" => SubscriptionStatus::Paused,
            "trialing" => SubscriptionStatus::Trialing,
            "unpaid" => SubscriptionStatus::Unpaid,
            _ => return Err(SubscriptionStatusError::InvalidValue),
        };

        Ok(variant)
    }
}

impl Decode<'_, Sqlite> for SubscriptionStatus {
    fn decode(value: SqliteValueRef<'_>) -> Result<Self, BoxDynError> {
        let inner_val = <&str as Decode<Sqlite>>::decode(value)?;
        Self::try_from(inner_val).map_err(Into::into)
    }
}

impl Encode<'_, Sqlite> for SubscriptionStatus {
    fn encode_by_ref(&self, args: &mut Vec<SqliteArgumentValue<'_>>) -> IsNull {
        args.push(SqliteArgumentValue::Text(self.to_string().into()));
        IsNull::No
    }
}

impl Type<Sqlite> for SubscriptionStatus {
    fn compatible(ty: &SqliteTypeInfo) -> bool {
        <&str as Type<Sqlite>>::compatible(ty)
    }

    fn type_info() -> SqliteTypeInfo {
        <&str as Type<Sqlite>>::type_info()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SubscriptionStatusError {
    #[error("attempted to decode unknown status value")]
    InvalidValue,
}
