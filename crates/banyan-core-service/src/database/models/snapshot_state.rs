use std::fmt;
use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};
use sqlx::encode::IsNull;
use sqlx::error::BoxDynError;
use sqlx::sqlite::{SqliteArgumentValue, SqliteTypeInfo, SqliteValueRef};
use sqlx::{Decode, Encode, Sqlite, Type};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
pub enum SnapshotState {
    Pending,
    Completed,
    Error,
}

impl From<String> for SnapshotState {
    fn from(s: String) -> Self {
        match s.as_str() {
            "pending" => SnapshotState::Pending,
            "completed" => SnapshotState::Completed,
            "error" => SnapshotState::Error,
            _ => panic!("invalid snapshot state"),
        }
    }
}

impl TryFrom<&str> for SnapshotState {
    type Error = SnapshotStateError;

    fn try_from(val: &str) -> Result<Self, SnapshotStateError> {
        let variant = match val {
            "pending" => SnapshotState::Pending,
            "completed" => SnapshotState::Completed,
            "error" => SnapshotState::Error,
            _ => return Err(SnapshotStateError::InvalidStateValue),
        };

        Ok(variant)
    }
}

impl Display for SnapshotState {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            SnapshotState::Pending => f.write_str("pending"),
            SnapshotState::Completed => f.write_str("completed"),
            SnapshotState::Error => f.write_str("error"),
        }
    }
}

impl Decode<'_, Sqlite> for SnapshotState {
    fn decode(value: SqliteValueRef<'_>) -> Result<Self, BoxDynError> {
        let inner_val = <&str as Decode<Sqlite>>::decode(value)?;
        Self::try_from(inner_val).map_err(Into::into)
    }
}

impl Encode<'_, Sqlite> for SnapshotState {
    fn encode_by_ref(&self, args: &mut Vec<SqliteArgumentValue<'_>>) -> IsNull {
        args.push(SqliteArgumentValue::Text(self.to_string().into()));
        IsNull::No
    }
}

impl Type<Sqlite> for SnapshotState {
    fn compatible(ty: &SqliteTypeInfo) -> bool {
        <&str as Type<Sqlite>>::compatible(ty)
    }

    fn type_info() -> SqliteTypeInfo {
        <&str as Type<Sqlite>>::type_info()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SnapshotStateError {
    #[error("attempted to decode unknown state value")]
    InvalidStateValue,
}

#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    use super::*;

    proptest! {
        /// Show that any [`SnapshotState`] may be serialized, and then deserialized.
        #[test]
        fn snapshot_states_can_be_round_tripped(input in any::<SnapshotState>()) {
            let round_trip = input.to_string().as_str().try_into().unwrap();
            prop_assert_eq!(input, round_trip);
        }
    }
}
