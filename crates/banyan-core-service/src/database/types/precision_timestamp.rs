use std::convert::TryFrom;
use std::fmt::{self, Display, Formatter};
use std::ops::Deref;

use serde::{Deserialize, Serialize};
use sqlx::encode::IsNull;
use sqlx::error::BoxDynError;
use sqlx::sqlite::{SqliteArgumentValue, SqliteTypeInfo, SqliteValueRef};
use sqlx::{Decode, Encode, Sqlite, Type};
use time::serde::iso8601;
use time::{format_description, OffsetDateTime};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrecisionTimestamp(#[serde(with = "iso8601")] OffsetDateTime);

impl PrecisionTimestamp {
    pub fn now_utc() -> Self {
        Self(OffsetDateTime::now_utc())
    }

    pub fn into_inner(self) -> OffsetDateTime {
        self.0
    }
}

impl From<OffsetDateTime> for PrecisionTimestamp {
    fn from(val: OffsetDateTime) -> Self {
        Self(val)
    }
}

impl Deref for PrecisionTimestamp {
    type Target = OffsetDateTime;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Decode<'_, Sqlite> for PrecisionTimestamp {
    fn decode(value: SqliteValueRef<'_>) -> Result<Self, BoxDynError> {
        let inner_val = <&str as Decode<Sqlite>>::decode(value)?;
        Self::try_from(inner_val).map_err(Into::into)
    }
}

impl Encode<'_, Sqlite> for PrecisionTimestamp {
    fn encode_by_ref(&self, args: &mut Vec<SqliteArgumentValue<'_>>) -> IsNull {
        args.push(SqliteArgumentValue::Text(self.to_string().into()));
        IsNull::No
    }
}

impl Type<Sqlite> for PrecisionTimestamp {
    fn compatible(ty: &SqliteTypeInfo) -> bool {
        <&str as Type<Sqlite>>::compatible(ty)
    }

    fn type_info() -> SqliteTypeInfo {
        <&str as Type<Sqlite>>::type_info()
    }
}

impl Display for PrecisionTimestamp {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // Note: always using 3 subsecond digits when writing to the database
        let format_description = format_description::parse(
            "[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:3]",
        )
        .map_err(|_| fmt::Error)?;
        let formatted = self.0.format(&format_description).map_err(|_| fmt::Error)?;
        write!(f, "{}", formatted)
    }
}

impl TryFrom<&str> for PrecisionTimestamp {
    type Error = PrecisionTimestampError;

    fn try_from(val: &str) -> Result<Self, PrecisionTimestampError> {
        // Note: we've not been encoding subseconds, so we gotta try and match for seconds first
        //  Match the format description to the length of the input string
        // Note: the format description for OffsetDateTime relies on the presence of an offset,
        //  which we don't have, so we'll just add one later
        let val_len = val.len();
        let format = match val_len {
            19 => {
                "[year]-[month]-[day] [hour]:[minute]:[second] [offset_hour \
            sign:mandatory]:[offset_minute]:[offset_second]"
            }
            23 => {
                "[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:3] [offset_hour \
            sign:mandatory]:[offset_minute]:[offset_second]"
            }
            _ => {
                return Err(PrecisionTimestampError::ParseTimestampInvalidLength(
                    val_len,
                ))
            }
        };
        let format = format_description::parse(format)?;

        // Add a zero offset to the end of the string
        let val = &format!("{} +00:00:00", val);

        let parsed = OffsetDateTime::parse(val, &format)?;

        Ok(Self(parsed))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PrecisionTimestampError {
    #[error("timestamp was incorrect size: {0}")]
    ParseTimestampInvalidLength(usize),
    #[error("invalid format description: {0}")]
    ParseTimestampInvalidFormatDescription(#[from] time::error::InvalidFormatDescription),
    #[error("could not parse timestamp: {0}")]
    ParseTimestamp(#[from] time::error::Parse),
    #[error("could not format timestamp: {0}")]
    FormatTimestamp(#[from] time::error::Format),
}

#[cfg(test)]
mod test {
    use std::error::Error;

    use time::macros::datetime;

    use super::*;
    use crate::database::test_helpers::setup_database;

    // It's too annoying to try and access Sqlx's internal types for these tests,
    // but this is a solid enough way to test crossing the database boundary

    #[tokio::test]
    async fn sqlx_timestamp_seconds_decoding() {
        let db_pool = setup_database().await;
        let mut transact = db_pool.begin().await.expect("transaction");

        // Epoch time
        let expected_timestamp = PrecisionTimestamp(datetime!(1970-01-01 0:00:00.000 UTC));

        let decoded_timestamp: PrecisionTimestamp = sqlx::query_scalar!(
            "SELECT CAST('1970-01-01 00:00:00' AS TEXT) as 'timestamp: PrecisionTimestamp';"
        )
        .fetch_one(&mut *transact)
        .await
        .expect("decode to succeed");
        assert_eq!(decoded_timestamp, expected_timestamp);

        #[derive(sqlx::FromRow)]
        struct PrecisionTimestampTest {
            timestamp: PrecisionTimestamp,
        }

        let decoded_obj = sqlx::query_as!(
            PrecisionTimestampTest,
            "SELECT CAST('1970-01-01 00:00:00' AS TEXT) as 'timestamp: PrecisionTimestamp';"
        )
        .fetch_one(&mut *transact)
        .await
        .expect("decode to succeed");
        assert_eq!(decoded_obj.timestamp, expected_timestamp);
        transact.rollback().await.expect("rollback")
    }

    #[tokio::test]
    async fn sqlx_timestamp_subseconds_decoding() {
        let db_pool = setup_database().await;
        let mut transact = db_pool.begin().await.expect("transaction");

        // Epoch time + 1 millisecond
        let expected_timestamp = PrecisionTimestamp(datetime!(1970-01-01 0:00:00.001 UTC));

        let decoded_timestamp: PrecisionTimestamp = sqlx::query_scalar!(
            "SELECT CAST('1970-01-01 00:00:00.001' AS TEXT) as 'timestamp: PrecisionTimestamp';"
        )
        .fetch_one(&mut *transact)
        .await
        .expect("decode to succeed");
        assert_eq!(decoded_timestamp, expected_timestamp);

        #[derive(sqlx::FromRow)]
        struct PrecisionTimestampTest {
            timestamp: PrecisionTimestamp,
        }

        let decoded_obj = sqlx::query_as!(
            PrecisionTimestampTest,
            "SELECT CAST('1970-01-01 00:00:00.001' AS TEXT) as 'timestamp: PrecisionTimestamp';"
        )
        .fetch_one(&mut *transact)
        .await
        .expect("decode to succeed");
        assert_eq!(decoded_obj.timestamp, expected_timestamp);
        transact.rollback().await.expect("rollback")
    }

    #[tokio::test]
    async fn sqlx_timestamp_decoding_failures() {
        let db_pool = setup_database().await;
        let mut transact = db_pool.begin().await.expect("transaction");

        let short_len_result = sqlx::query_scalar!(
            "SELECT CAST('1970-01-01 0:00:00' AS TEXT) as 'timestamp: PrecisionTimestamp';"
        )
        .fetch_one(&mut *transact)
        .await;

        assert!(short_len_result.is_err());

        let err = short_len_result.unwrap_err();
        assert!(matches!(err, sqlx::Error::ColumnDecode { .. }));

        let inner_err = err.source().expect("a source");
        let timestamp_error = inner_err
            .downcast_ref::<PrecisionTimestampError>()
            .expect("error to be ours");
        assert!(matches!(
            timestamp_error,
            PrecisionTimestampError::ParseTimestampInvalidLength(_)
        ));

        let long_len_result = sqlx::query_scalar!(
            "SELECT CAST('1970-01-01 00:00:00.001000' AS TEXT) as 'timestamp: PrecisionTimestamp';"
        )
        .fetch_one(&mut *transact)
        .await;

        assert!(long_len_result.is_err());

        let err = long_len_result.unwrap_err();
        assert!(matches!(err, sqlx::Error::ColumnDecode { .. }));

        let inner_err = err.source().expect("a source");
        let timestamp_error = inner_err
            .downcast_ref::<PrecisionTimestampError>()
            .expect("error to be ours");
        assert!(matches!(
            timestamp_error,
            PrecisionTimestampError::ParseTimestampInvalidLength(_)
        ));

        let right_len_bad_text = sqlx::query_scalar!(
            "SELECT CAST('not   a   timestamp' AS TEXT) as 'timestamp: PrecisionTimestamp';"
        )
        .fetch_one(&mut *transact)
        .await;

        assert!(right_len_bad_text.is_err());

        let err = right_len_bad_text.unwrap_err();
        assert!(matches!(err, sqlx::Error::ColumnDecode { .. }));

        let inner_err = err.source().expect("a source");
        let timestamp_error = inner_err
            .downcast_ref::<PrecisionTimestampError>()
            .expect("error to be ours");
        assert!(matches!(
            timestamp_error,
            PrecisionTimestampError::ParseTimestamp(_)
        ));

        transact.rollback().await.expect("rollback")
    }

    #[tokio::test]
    async fn sqlx_timestamp_encoding() {
        let db_pool = setup_database().await;
        let mut transact = db_pool.begin().await.expect("transaction");

        sqlx::query("CREATE TABLE timestamp_encoding_test (timestamp TEXT NOT NULL);")
            .execute(&mut *transact)
            .await
            .expect("setup to succeed");

        // Epoch time + 1 nanosecond
        let sample_timestamp = PrecisionTimestamp(datetime!(1970-01-01 0:00:00.001 UTC));

        let returned_timestamp: PrecisionTimestamp = sqlx::query_scalar(
            "INSERT INTO timestamp_encoding_test (timestamp) VALUES ($1) RETURNING timestamp as 'timestamp: PrecisionTimestamp';",
        )
        .bind(sample_timestamp.clone())
        .fetch_one(&mut *transact)
        .await
        .expect("insert to succeed");

        assert_eq!(sample_timestamp, returned_timestamp);

        let raw_timestamp: String =
            sqlx::query_scalar("SELECT timestamp FROM timestamp_encoding_test;")
                .fetch_one(&mut *transact)
                .await
                .expect("return to succeed");

        assert_eq!(&raw_timestamp, sample_timestamp.to_string().as_str());
    }
}
