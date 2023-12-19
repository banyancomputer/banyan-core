use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

use sqlx::encode::IsNull;
use sqlx::error::BoxDynError;
use sqlx::sqlite::{SqliteArgumentValue, SqliteTypeInfo, SqliteValueRef};
use sqlx::{Decode, Encode, Sqlite, Type};
use time::OffsetDateTime;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatabaseTimestamp(OffsetDateTime);

impl Decode<'_, Sqlite> for DatabaseTimestamp {
    fn decode(value: SqliteValueRef<'_>) -> Result<Self, BoxDynError> {
        let inner_val = <&str as Decode<Sqlite>>::decode(value)?;
        Self::try_from(inner_val).map_err(Into::into)
    }
}

impl Encode<'_, Sqlite> for DatabaseTimestamp {
    fn encode_by_ref(&self, args: &mut Vec<SqliteArgumentValue<'_>>) -> IsNull {
        args.push(SqliteArgumentValue::Text(self.to_string().into()));
        IsNull::No
    }
}

impl Type<Sqlite> for DatabaseTimestamp {
    fn compatible(ty: &SqliteTypeInfo) -> bool {
        <&str as Type<Sqlite>>::compatible(ty)
    }

    fn type_info() -> SqliteTypeInfo {
        <&str as Type<Sqlite>>::type_info()
    }
}

impl Display for DatabaseTimestamp {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0.unix_timestamp_nanos().to_string())
    }
}

impl TryFrom<&str> for DatabaseTimestamp {
    type Error = DatabaseTimestampError;

    fn try_from(val: &str) -> Result<Self, DatabaseTimestampError> {
        let offset_date_time_int: i128 = i128::from_str(val)?;
        let offset_date_time = OffsetDateTime::from_unix_timestamp_nanos(offset_date_time_int)?;
        Ok(Self(offset_date_time))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DatabaseTimestampError {
    #[error("could not parse i128 from timestamp text: {0}")]
    ParseInt(#[from] std::num::ParseIntError),
    #[error("could not parse unix timestamp: {0}")]
    ParseDatabaseTimestamp(#[from] time::error::ComponentRange),
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
    async fn sqlx_timestamp_decoding() {
        let db_pool = setup_database().await;
        let mut transact = db_pool.begin().await.expect("transaction");

        // Epoch time + 1 nanosecond
        let expected_timestamp = DatabaseTimestamp(datetime!(1970-01-01 0:00:00.000_000_001 UTC));

        let decoded_timestamp: DatabaseTimestamp =
            sqlx::query_scalar!("SELECT CAST('1' AS TEXT) as 'timestamp: DatabaseTimestamp';")
                .fetch_one(&mut *transact)
                .await
                .expect("decode to succeed");
        assert_eq!(decoded_timestamp, expected_timestamp);

        #[derive(sqlx::FromRow)]
        struct DatabaseTimestampTest {
            timestamp: DatabaseTimestamp,
        }

        let decoded_obj = sqlx::query_as!(
            DatabaseTimestampTest,
            "SELECT CAST('1' AS TEXT) as 'timestamp: DatabaseTimestamp'"
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

        let bad_int_result = sqlx::query_scalar!(
            "SELECT CAST('candy corn is tast' AS TEXT) as 'timestamp: DatabaseTimestamp';"
        )
        .fetch_one(&mut *transact)
        .await;

        assert!(bad_int_result.is_err());

        let err = bad_int_result.unwrap_err();
        assert!(matches!(err, sqlx::Error::ColumnDecode { .. }));

        let inner_err = err.source().expect("a source");
        let timestamp_error = inner_err
            .downcast_ref::<DatabaseTimestampError>()
            .expect("error to be ours");
        assert!(matches!(
            timestamp_error,
            DatabaseTimestampError::ParseInt(_)
        ));

        // TODO: I think if we have a valid instance of i128, there's no reason
        // from_unix_timestamp_nanos should fail ... but i should verify this

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
        let sample_timestamp = DatabaseTimestamp(datetime!(1970-01-01 0:00:00.000_000_001 UTC));

        let returned_timestamp: DatabaseTimestamp = sqlx::query_scalar(
            "INSERT INTO timestamp_encoding_test (timestamp) VALUES ($1) RETURNING timestamp as 'timestamp: DatabaseTimestamp';",
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

        assert_eq!(&raw_timestamp, "1");
    }
}
