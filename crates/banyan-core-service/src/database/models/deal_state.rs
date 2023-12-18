use std::fmt::Display;

use serde::{Deserialize, Serialize};
use sqlx::encode::IsNull;
use sqlx::error::BoxDynError;
use sqlx::sqlite::{SqliteArgumentValue, SqliteTypeInfo, SqliteValueRef};
use sqlx::{Decode, Encode, Sqlite, Type};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
pub enum DealState {
    Active,
    Accepted,
    Sealed,
    Finalized,
    Cancelled,
}

impl From<String> for DealState {
    fn from(s: String) -> Self {
        match s.as_str() {
            "active" => DealState::Active,
            "accepted" => DealState::Accepted,
            "sealed" => DealState::Sealed,
            "finalized" => DealState::Finalized,
            "cancelled" => DealState::Cancelled,
            _ => panic!("invalid deal state"),
        }
    }
}

impl TryFrom<&str> for DealState {
    type Error = DealStateError;

    fn try_from(val: &str) -> Result<Self, DealStateError> {
        let variant = match val {
            "active" => DealState::Active,
            "accepted" => DealState::Accepted,
            "sealed" => DealState::Sealed,
            "finalized" => DealState::Finalized,
            "cancelled" => DealState::Cancelled,
            _ => return Err(DealStateError::InvalidStateValue),
        };

        Ok(variant)
    }
}

impl Display for DealState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                DealState::Active => "active",
                DealState::Accepted => "accepted",
                DealState::Sealed => "sealed",
                DealState::Finalized => "finalized",
                DealState::Cancelled => "cancelled",
            }
        )
    }
}

impl Decode<'_, Sqlite> for DealState {
    fn decode(value: SqliteValueRef<'_>) -> Result<Self, BoxDynError> {
        let inner_val = <&str as Decode<Sqlite>>::decode(value)?;
        Self::try_from(inner_val).map_err(Into::into)
    }
}

impl Encode<'_, Sqlite> for DealState {
    fn encode_by_ref(&self, args: &mut Vec<SqliteArgumentValue<'_>>) -> IsNull {
        args.push(SqliteArgumentValue::Text(self.to_string().into()));
        IsNull::No
    }
}

impl Type<Sqlite> for DealState {
    fn compatible(ty: &SqliteTypeInfo) -> bool {
        <&str as Type<Sqlite>>::compatible(ty)
    }

    fn type_info() -> SqliteTypeInfo {
        <&str as Type<Sqlite>>::type_info()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DealStateError {
    #[error("attempted to decode unknown state value")]
    InvalidStateValue,
}

#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    use super::*;

    proptest! {
        /// Show that any [`DealState`] may be serialized, and then deserialized.
        #[test]
        fn deal_states_can_be_round_tripped(input in any::<DealState>()) {
            let round_trip = input.to_string().as_str().try_into().unwrap();
            prop_assert_eq!(input, round_trip);
        }
    }
}
