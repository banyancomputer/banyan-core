use std::fmt::Display;

use serde::{Deserialize, Serialize};
use sqlx::encode::IsNull;
use sqlx::error::BoxDynError;
use sqlx::sqlite::{SqliteArgumentValue, SqliteTypeInfo, SqliteValueRef};
use sqlx::{Decode, Encode, Sqlite, Type};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, Default)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
pub enum BlockLocationState {
    #[default]
    SyncRequired,
    Staged,
    Stable,
}

impl From<String> for BlockLocationState {
    fn from(s: String) -> Self {
        match s.as_str() {
            "sync_required" => BlockLocationState::SyncRequired,
            "staged" => BlockLocationState::Staged,
            "stable" => BlockLocationState::Stable,
            _ => panic!("invalid block location state"),
        }
    }
}

impl TryFrom<&str> for BlockLocationState {
    type Error = BlockLocationStateError;

    fn try_from(val: &str) -> Result<Self, BlockLocationStateError> {
        let variant = match val {
            "sync_required" => BlockLocationState::SyncRequired,
            "staged" => BlockLocationState::Staged,
            "stable" => BlockLocationState::Stable,
            _ => return Err(BlockLocationStateError::InvalidStateValue),
        };

        Ok(variant)
    }
}

impl Display for BlockLocationState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                BlockLocationState::SyncRequired => "sync_required",
                BlockLocationState::Staged => "staged",
                BlockLocationState::Stable => "stable",
            }
        )
    }
}

impl Decode<'_, Sqlite> for BlockLocationState {
    fn decode(value: SqliteValueRef<'_>) -> Result<Self, BoxDynError> {
        let inner_val = <&str as Decode<Sqlite>>::decode(value)?;
        Self::try_from(inner_val).map_err(Into::into)
    }
}

impl Encode<'_, Sqlite> for BlockLocationState {
    fn encode_by_ref(&self, args: &mut Vec<SqliteArgumentValue<'_>>) -> IsNull {
        args.push(SqliteArgumentValue::Text(self.to_string().into()));
        IsNull::No
    }
}

impl Type<Sqlite> for BlockLocationState {
    fn compatible(ty: &SqliteTypeInfo) -> bool {
        <&str as Type<Sqlite>>::compatible(ty)
    }

    fn type_info() -> SqliteTypeInfo {
        <&str as Type<Sqlite>>::type_info()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum BlockLocationStateError {
    #[error("attempted to decode unknown state value")]
    InvalidStateValue,
}

#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    use super::*;

    proptest! {
        /// Show that any [`BlockLocationState`] may be serialized, and then deserialized.
        #[test]
        fn deal_states_can_be_round_tripped(input in any::<BlockLocationState>()) {
            let round_trip = input.to_string().as_str().try_into().unwrap();
            prop_assert_eq!(input, round_trip);
        }
    }
}
