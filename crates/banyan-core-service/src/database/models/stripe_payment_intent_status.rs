use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};
use sqlx::encode::IsNull;
use sqlx::error::BoxDynError;
use sqlx::sqlite::{SqliteArgumentValue, SqliteTypeInfo, SqliteValueRef};
use sqlx::{Decode, Encode, Sqlite, Type};

use stripe::generated::core::payment_intent::PaymentIntentStatus as StripeLibPaymentIntentStatus;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StripePaymentIntentStatus {
    RequiresPaymentMethod,
    RequiresConfirmation,
    RequiresAction,
    Processing,
    RequiresCapture,
    Canceled,
    Succeeded,
}

impl Display for StripePaymentIntentStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            StripePaymentIntentStatus::RequiresPaymentMethod => f.write_str("requires_payment_method"),
            StripePaymentIntentStatus::RequiresConfirmation => f.write_str("requires_confirmation"),
            StripePaymentIntentStatus::RequiresAction => f.write_str("requires_action"),
            StripePaymentIntentStatus::Processing => f.write_str("processing"),
            StripePaymentIntentStatus::RequiresCapture => f.write_str("requires_capture"),
            StripePaymentIntentStatus::Canceled => f.write_str("canceled"),
            StripePaymentIntentStatus::Succeeded => f.write_str("succeeded"),
        }
    }
}

impl From<StripeLibPaymentIntentStatus> for StripePaymentIntentStatus {
    fn from(val: StripeLibPaymentIntentStatus) -> Self {
        match val {
            StripeLibPaymentIntentStatus::Canceled => StripePaymentIntentStatus::Canceled,
            StripeLibPaymentIntentStatus::Processing => StripePaymentIntentStatus::Processing,
            StripeLibPaymentIntentStatus::RequiresAction => StripePaymentIntentStatus::RequiresAction,
            StripeLibPaymentIntentStatus::RequiresCapture => StripePaymentIntentStatus::RequiresCapture,
            StripeLibPaymentIntentStatus::RequiresConfirmation => StripePaymentIntentStatus::RequiresConfirmation,
            StripeLibPaymentIntentStatus::RequiresPaymentMethod => StripePaymentIntentStatus::RequiresPaymentMethod,
            StripeLibPaymentIntentStatus::Succeeded => StripePaymentIntentStatus::Succeeded,
        }
    }
}

impl TryFrom<&str> for StripePaymentIntentStatus {
    type Error = StripePaymentIntentStatusError;

    fn try_from(val: &str) -> Result<Self, StripePaymentIntentStatusError> {
        let variant = match val {
            "requires_payment_method" => StripePaymentIntentStatus::RequiresPaymentMethod,
            "requires_confirmation" => StripePaymentIntentStatus::RequiresConfirmation,
            "requires_action" => StripePaymentIntentStatus::RequiresAction,
            "processing" => StripePaymentIntentStatus::Processing,
            "requires_capture" => StripePaymentIntentStatus::RequiresCapture,
            "canceled" => StripePaymentIntentStatus::Canceled,
            "succeeded" => StripePaymentIntentStatus::Succeeded,
            _ => return Err(StripePaymentIntentStatusError::InvalidValue),
        };

        Ok(variant)
    }
}

impl Decode<'_, Sqlite> for StripePaymentIntentStatus {
    fn decode(value: SqliteValueRef<'_>) -> Result<Self, BoxDynError> {
        let inner_val = <&str as Decode<Sqlite>>::decode(value)?;
        Self::try_from(inner_val).map_err(Into::into)
    }
}

impl Encode<'_, Sqlite> for StripePaymentIntentStatus {
    fn encode_by_ref(&self, args: &mut Vec<SqliteArgumentValue<'_>>) -> IsNull {
        args.push(SqliteArgumentValue::Text(self.to_string().into()));
        IsNull::No
    }
}

impl Type<Sqlite> for StripePaymentIntentStatus {
    fn compatible(ty: &SqliteTypeInfo) -> bool {
        <&str as Type<Sqlite>>::compatible(ty)
    }

    fn type_info() -> SqliteTypeInfo {
        <&str as Type<Sqlite>>::type_info()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum StripePaymentIntentStatusError {
    #[error("attempted to decode unknown status value")]
    InvalidValue,
}
