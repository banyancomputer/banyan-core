use serde::{Deserialize, Serialize};
use sqlx::encode::IsNull;
use sqlx::error::BoxDynError;
use sqlx::sqlite::{SqliteArgumentValue, SqliteTypeInfo, SqliteValueRef};
use sqlx::{Decode, Encode, Sqlite, Type};
use std::fmt::{self, Display, Formatter};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
pub enum NotificationSeverity {
    Warning,
    Error,
}

impl Display for NotificationSeverity {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            NotificationSeverity::Warning => f.write_str("warning"),
            NotificationSeverity::Error => f.write_str("error"),
        }
    }
}

impl TryFrom<&str> for NotificationSeverity {
    type Error = NotificationSeverityError;

    fn try_from(val: &str) -> Result<Self, NotificationSeverityError> {
        let variant = match val {
            "warning" => NotificationSeverity::Warning,
            "error" => NotificationSeverity::Error,
            _ => return Err(NotificationSeverityError::InvalidStateValue),
        };

        Ok(variant)
    }
}

impl Decode<'_, Sqlite> for NotificationSeverity {
    fn decode(value: SqliteValueRef<'_>) -> Result<Self, BoxDynError> {
        let inner_val = <&str as Decode<Sqlite>>::decode(value)?;
        Self::try_from(inner_val).map_err(Into::into)
    }
}

impl Encode<'_, Sqlite> for NotificationSeverity {
    fn encode_by_ref(&self, args: &mut Vec<SqliteArgumentValue<'_>>) -> IsNull {
        args.push(SqliteArgumentValue::Text(self.to_string().into()));
        IsNull::No
    }
}

impl Type<Sqlite> for NotificationSeverity {
    fn compatible(ty: &SqliteTypeInfo) -> bool {
        <&str as Type<Sqlite>>::compatible(ty)
    }

    fn type_info() -> SqliteTypeInfo {
        <&str as Type<Sqlite>>::type_info()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum NotificationSeverityError {
    #[error("attempted to decode unknown state value")]
    InvalidStateValue,
}

#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    use super::*;

    proptest! {
        /// Show that any [`NotificationSeverity`] may be serialized, and then deserialized.
        #[test]
        fn notification_severities_can_be_round_tripped(input in any::<NotificationSeverity>()) {
            let round_trip = input.to_string().as_str().try_into().unwrap();
            prop_assert_eq!(input, round_trip);
        }
    }
}
