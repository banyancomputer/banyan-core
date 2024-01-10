use crate::database::models::ExplicitBigInt;
use crate::database::DatabaseConnection;

/// A partial version of a storage host encompassing only the data needed for clients that need to
/// send data to the storage host.
#[derive(sqlx::FromRow)]
pub struct SelectedStorageHost {
    pub id: String,
    pub name: String,
    pub url: String,
    pub used_storage: i64,
    pub available_storage: i64,
    pub fingerprint: String,
    pub pem: String,
}

impl SelectedStorageHost {
    /// Find the database ID of a storage host that has the requested capacity currently available.
    /// Will return None if no storage host has the requested capacity available. Does not prefer
    /// any storage host over any other.
    pub async fn select_for_capacity(
        conn: &mut DatabaseConnection,
        required_bytes: i64,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as!(
            Self,
            r#"SELECT id,name,url,used_storage, available_storage,fingerprint,pem FROM storage_hosts
                   WHERE (available_storage - used_storage) > $1
                   ORDER BY RANDOM()
                   LIMIT 1;"#,
            required_bytes,
        )
        .fetch_optional(&mut *conn)
        .await
    }
}

pub struct StorageHost;

impl StorageHost {
    /// Retrieves the current known amount of data owned by a particular user that is located at
    /// the requested storage provider as well the reservation capacity the user currently has at
    /// that storage provider if any.
    pub async fn user_report(
        conn: &mut DatabaseConnection,
        storage_host_id: &str,
        user_id: &str,
    ) -> Result<UserStorageReport, sqlx::Error> {
        let ex_bigint = sqlx::query_as!(
            ExplicitBigInt,
            r#"SELECT COALESCE(SUM(m.data_size), 0) as big_int FROM metadata m
                   JOIN storage_hosts_metadatas_storage_grants shmg ON shmg.metadata_id = m.id
                   JOIN storage_grants sg ON shmg.storage_grant_id = sg.id
                   WHERE shmg.storage_host_id = $2
                       AND sg.user_id = $1;
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
