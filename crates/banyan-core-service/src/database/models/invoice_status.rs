use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};
use sqlx::encode::IsNull;
use sqlx::error::BoxDynError;
use sqlx::sqlite::{SqliteArgumentValue, SqliteTypeInfo, SqliteValueRef};
use sqlx::{Decode, Encode, Sqlite, Type};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InvoiceStatus {
    Draft,
    Open,
    Paid,
    Uncollectible,
    Void,
}

impl Display for InvoiceStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            InvoiceStatus::Draft => f.write_str("draft"),
            InvoiceStatus::Open => f.write_str("open"),
            InvoiceStatus::Paid => f.write_str("paid"),
            InvoiceStatus::Uncollectible => f.write_str("uncollectible"),
            InvoiceStatus::Void => f.write_str("voice"),
        }
    }
}

use stripe::generated::billing::invoice::InvoiceStatus as StripeInvoiceStatus;

impl From<StripeInvoiceStatus> for InvoiceStatus {
    fn from(val: StripeInvoiceStatus) -> Self {
        match val {
            StripeInvoiceStatus::Draft => InvoiceStatus::Draft,
            StripeInvoiceStatus::Open => InvoiceStatus::Open,
            StripeInvoiceStatus::Paid => InvoiceStatus::Paid,
            StripeInvoiceStatus::Uncollectible => InvoiceStatus::Uncollectible,
            StripeInvoiceStatus::Void => InvoiceStatus::Void,
        }
    }
}

impl TryFrom<&str> for InvoiceStatus {
    type Error = InvoiceStatusError;

    fn try_from(val: &str) -> Result<Self, InvoiceStatusError> {
        let variant = match val {
            "draft" => InvoiceStatus::Draft,
            "open" => InvoiceStatus::Open,
            "paid" => InvoiceStatus::Paid,
            "uncollectible" => InvoiceStatus::Uncollectible,
            "voice" => InvoiceStatus::Void,
            _ => return Err(InvoiceStatusError::InvalidValue),
        };

        Ok(variant)
    }
}

impl Decode<'_, Sqlite> for InvoiceStatus {
    fn decode(value: SqliteValueRef<'_>) -> Result<Self, BoxDynError> {
        let inner_val = <&str as Decode<Sqlite>>::decode(value)?;
        Self::try_from(inner_val).map_err(Into::into)
    }
}

impl Encode<'_, Sqlite> for InvoiceStatus {
    fn encode_by_ref(&self, args: &mut Vec<SqliteArgumentValue<'_>>) -> IsNull {
        args.push(SqliteArgumentValue::Text(self.to_string().into()));
        IsNull::No
    }
}

impl Type<Sqlite> for InvoiceStatus {
    fn compatible(ty: &SqliteTypeInfo) -> bool {
        <&str as Type<Sqlite>>::compatible(ty)
    }

    fn type_info() -> SqliteTypeInfo {
        <&str as Type<Sqlite>>::type_info()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum InvoiceStatusError {
    #[error("attempted to decode unknown status value")]
    InvalidValue,
}
