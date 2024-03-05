use time::OffsetDateTime;

use crate::database::models::ExplicitBigInt;
use crate::database::{Database, DatabaseConnection};

/// A partial version of a storage host encompassing only the data needed for clients that need to
/// send data to the storage host.
#[derive(sqlx::FromRow)]
pub struct StorageHost {
    pub id: String,
    pub name: String,
    pub url: String,
    pub used_storage: i64,
    pub reserved_storage: i64,
    pub available_storage: i64,
    pub staging: bool,
    pub region: Option<String>,
    pub last_seen_at: Option<OffsetDateTime>,
    pub current_version: Option<String>,
    pub fingerprint: String,
    pub pem: String,
}

impl StorageHost {
    /// Find the database ID of a storage host that has the requested capacity currently available.
    /// Will return None if there is no storage host with the requested capacity and region
    /// available, but does not exert preference among hosts that meet these criteria.
    pub async fn select_for_capacity(
        conn: &mut DatabaseConnection,
        region_preference: Option<String>,
        required_bytes: i64,
    ) -> Result<Option<Self>, sqlx::Error> {
        // Select a storage host with enough free space, ensuring it is also within the region if
        // one is specified.
        let region_specific_host: Option<Self> = sqlx::query_as!(
            Self,
            r#"
                SELECT *
                FROM storage_hosts
                WHERE (available_storage- reserved_storage) > $1
                AND ($2 IS NULL OR $2 LIKE ('%' || region || '%'))
                ORDER BY RANDOM()
                LIMIT 1;
            "#,
            required_bytes,
            region_preference,
        )
        .fetch_optional(&mut *conn)
        .await?;

        if region_specific_host.is_some() {
            Ok(region_specific_host)
        } else {
            sqlx::query_as!(
                Self,
                r#"
                    SELECT *
                    FROM storage_hosts
                    WHERE (available_storage - reserved_storage) > $1
                    ORDER BY RANDOM()
                    LIMIT 1;
                "#,
                required_bytes,
            )
            .fetch_optional(&mut *conn)
            .await
        }
    }

    pub async fn select_for_capacity_with_exclusion(
        database: &Database,
        required_bytes: i64,
        exclude_host_id: &str,
    ) -> Result<Self, sqlx::Error> {
        sqlx::query_as!(
            Self,
            r#"SELECT * FROM storage_hosts
                       WHERE (available_storage - used_storage) > $1
                       AND id != $2
                       ORDER BY RANDOM()
                       LIMIT 1;"#,
            required_bytes,
            exclude_host_id
        )
        .fetch_one(database)
        .await
    }

    pub async fn select_staging(conn: &Database) -> Result<Self, sqlx::Error> {
        sqlx::query_as!(
            Self,
            r#"SELECT * FROM storage_hosts WHERE staging IS TRUE;"#,
        )
        .fetch_one(conn)
        .await
    }

    pub async fn find_by_id(conn: &Database, id: &str) -> Result<Self, sqlx::Error> {
        sqlx::query_as!(Self, "SELECT * FROM storage_hosts WHERE id = $1;", id,)
            .fetch_one(conn)
            .await
    }

    pub async fn find_by_id_with_transaction(
        conn: &mut DatabaseConnection,
        id: &str,
    ) -> Result<Self, sqlx::Error> {
        sqlx::query_as!(Self, "SELECT * FROM storage_hosts WHERE id = $1;", id,)
            .fetch_one(&mut *conn)
            .await
    }
}

/// Type representing the amount of data a particular user has stored at an individual storage host
/// as well as the maximum amount the same user is authorized to store there. The authorized amount
/// should always be > 0 otherwise that particular storage host will know nothing about the account
/// and the `user_report` method that generates this will return an error.
#[derive(Debug)]
pub struct UserStorageReport {
    current_consumption: i64,
    maximum_authorized: Option<i64>,
}

impl UserStorageReport {
    /// Retrieves the current known amount of data owned by a particular user that is located at
    /// the requested storage provider as well the reservation capacity the user currently has at
    /// that storage provider if any.
    pub async fn user_report(
        conn: &mut DatabaseConnection,
        storage_host_id: &str,
        user_id: &str,
    ) -> Result<Self, sqlx::Error> {
        let ex_bigint = sqlx::query_as!(
            ExplicitBigInt,
            r#"SELECT COALESCE(SUM(m.data_size), 0) as big_int FROM metadata m
                   JOIN storage_hosts_metadatas_storage_grants shmg ON shmg.metadata_id = m.id
                   JOIN storage_grants sg ON shmg.storage_grant_id = sg.id
                   WHERE shmg.storage_host_id = $1 AND sg.user_id = $2;
             "#,
            storage_host_id,
            user_id,
        )
        .fetch_one(&mut *conn)
        .await?;
        let current_consumption = ex_bigint.big_int;

        let maximum_authorized = sqlx::query_scalar!(
            r#"SELECT authorized_amount FROM storage_grants
                   WHERE storage_host_id = $1
                       AND user_id = $2
                       AND redeemed_at IS NOT NULL
                   ORDER BY created_at DESC
                   LIMIT 1;"#,
            storage_host_id,
            user_id,
        )
        .fetch_optional(&mut *conn)
        .await?;

        Ok(UserStorageReport {
            current_consumption,
            maximum_authorized,
        })
    }

    pub async fn total_consumption(
        conn: &mut DatabaseConnection,
        storage_host_id: &str,
    ) -> Result<i64, sqlx::Error> {
        let ex_bigint = sqlx::query_as!(
            ExplicitBigInt,
            r#"SELECT COALESCE(SUM(COALESCE(m.data_size, m.expected_data_size, 0)), 0) as big_int
                    FROM storage_hosts_metadatas_storage_grants shms
                    INNER JOIN metadata AS m ON m.id = shms.metadata_id
                    WHERE shms.storage_host_id = $1;
             "#,
            storage_host_id,
        )
        .fetch_one(&mut *conn)
        .await?;

        Ok(ex_bigint.big_int)
    }
}

    /// Provides the amount of storage a user has remaining on their authorization at the specific
    /// storage host. If the user has managed to go over their quota or they don't yet have an
    /// authorization at a storage host this will return 0.
    /// will never return a negative number.
    pub fn authorization_available(&self) -> i64 {
        match self.maximum_authorized {
            Some(ma) => (ma - self.current_consumption).max(0),
            None => 0,
        }
    }

    pub fn current_consumption(&self) -> i64 {
        self.current_consumption
    }
}
