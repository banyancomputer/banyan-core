use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};
use sqlx::encode::IsNull;
use sqlx::error::BoxDynError;
use sqlx::sqlite::{SqliteArgumentValue, SqliteTypeInfo, SqliteValueRef};
use sqlx::{Decode, Encode, Sqlite, Type};

use crate::pricing::{TAX_CODE_BUSINESS, TAX_CODE_PERSONAL};

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TaxClass {
    Business,
    NotApplicable,
    Personal,
}

impl TaxClass {
    pub fn stripe_id(&self) -> Option<&str> {
        match self {
            TaxClass::Personal => Some(TAX_CODE_PERSONAL),
            TaxClass::NotApplicable => None,
            TaxClass::Business => Some(TAX_CODE_BUSINESS),
        }
    }
}

impl Display for TaxClass {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            TaxClass::Business => f.write_str("business"),
            TaxClass::NotApplicable => f.write_str("not_applicable"),
            TaxClass::Personal => f.write_str("personal"),
        }
    }
}

impl TryFrom<&str> for TaxClass {
    type Error = TaxClassError;

    fn try_from(val: &str) -> Result<Self, TaxClassError> {
        let variant = match val {
            "business" => TaxClass::Business,
            "not_applicable" => TaxClass::NotApplicable,
            "personal" => TaxClass::Personal,
            _ => return Err(TaxClassError::InvalidValue),
        };

        Ok(variant)
    }
}

impl Decode<'_, Sqlite> for TaxClass {
    fn decode(value: SqliteValueRef<'_>) -> Result<Self, BoxDynError> {
        let inner_val = <&str as Decode<Sqlite>>::decode(value)?;
        Self::try_from(inner_val).map_err(Into::into)
    }
}

impl Encode<'_, Sqlite> for TaxClass {
    fn encode_by_ref(&self, args: &mut Vec<SqliteArgumentValue<'_>>) -> IsNull {
        args.push(SqliteArgumentValue::Text(self.to_string().into()));
        IsNull::No
    }
}

impl Type<Sqlite> for TaxClass {
    fn compatible(ty: &SqliteTypeInfo) -> bool {
        <&str as Type<Sqlite>>::compatible(ty)
    }

    fn type_info() -> SqliteTypeInfo {
        <&str as Type<Sqlite>>::type_info()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TaxClassError {
    #[error("attempted to decode unknown tax class")]
    InvalidValue,
}
