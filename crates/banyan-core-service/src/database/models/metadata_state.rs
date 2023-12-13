use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};
use sqlx::encode::IsNull;
use sqlx::error::BoxDynError;
use sqlx::sqlite::{SqliteArgumentValue, SqliteTypeInfo, SqliteValueRef};
use sqlx::{Decode, Encode, Sqlite, Type};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
pub enum MetadataState {
    Uploading,
    UploadFailed,
    Pending,
    Current,
    Outdated,
    Deleted,
}

impl Display for MetadataState {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            MetadataState::Uploading => f.write_str("uploading"),
            MetadataState::UploadFailed => f.write_str("upload_failed"),
            MetadataState::Pending => f.write_str("pending"),
            MetadataState::Current => f.write_str("current"),
            MetadataState::Outdated => f.write_str("outdated"),
            MetadataState::Deleted => f.write_str("deleted"),
        }
    }
}

impl TryFrom<&str> for MetadataState {
    type Error = MetadataStateError;

    fn try_from(val: &str) -> Result<Self, MetadataStateError> {
        let variant = match val {
            "uploading" => MetadataState::Uploading,
            "upload_failed" => MetadataState::UploadFailed,
            "pending" => MetadataState::Pending,
            "current" => MetadataState::Current,
            "outdated" => MetadataState::Outdated,
            "deleted" => MetadataState::Deleted,
            _ => return Err(MetadataStateError::InvalidStateValue),
        };

        Ok(variant)
    }
}

impl Decode<'_, Sqlite> for MetadataState {
    fn decode(value: SqliteValueRef<'_>) -> Result<Self, BoxDynError> {
        let inner_val = <&str as Decode<Sqlite>>::decode(value)?;
        Self::try_from(inner_val).map_err(Into::into)
    }
}

impl Encode<'_, Sqlite> for MetadataState {
    fn encode_by_ref(&self, args: &mut Vec<SqliteArgumentValue<'_>>) -> IsNull {
        args.push(SqliteArgumentValue::Text(self.to_string().into()));
        IsNull::No
    }
}

impl Type<Sqlite> for MetadataState {
    fn compatible(ty: &SqliteTypeInfo) -> bool {
        <&str as Type<Sqlite>>::compatible(ty)
    }

    fn type_info() -> SqliteTypeInfo {
        <&str as Type<Sqlite>>::type_info()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MetadataStateError {
    #[error("attempted to decode unknown state value")]
    InvalidStateValue,
}

#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    use super::*;

    proptest! {
        /// Show that any [`MetadataState`] may be serialized, and then deserialized.
        #[test]
        fn metadata_states_can_be_round_tripped(input in any::<MetadataState>()) {
            let round_trip = input.to_string().as_str().try_into().unwrap();
            prop_assert_eq!(input, round_trip);
        }
    }
}
