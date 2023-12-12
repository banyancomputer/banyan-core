use crate::database::DatabaseConnection;

pub struct StorageHost;

impl StorageHost {
    /// Find the database ID of a storage host that has the requested capacity currently available.
    /// Will return None if no storage host has the requested capacity available. Does not prefer
    /// any storage host over any other.
    pub async fn select_for_capacity(
        conn: &mut DatabaseConnection,
        required_bytes: i64,
    ) -> Result<Option<String>, sqlx::Error> {
        sqlx::query_scalar!(
            r#"SELECT id FROM storage_hosts
                   WHERE (available_storage - used_storage) > $1
                   ORDER BY RANDOM()
                   LIMIT 1;"#,
            required_bytes,
        )
        .fetch_optional(&mut *conn)
        .await
    }

    pub async fn user_report(
        _conn: &mut DatabaseConnection,
        _storage_host_id: &str,
        _user_id: &str,
    ) -> Result<UserStorageReport, sqlx::Error> {
        todo!()
    }
}

/// Type representing the amount of data a particular user has stored at an individual storage host
/// as well as the maximum amount the same user is authorized to store there. The authorized amount
/// should always be > 0 otherwise that particular storage host will know nothing about the account
/// and the `user_report` method that generates this will return an error.
#[derive(Debug)]
pub struct UserStorageReport {
    current_consumption: i64,
    maximum_authorized: i64,
}

impl UserStorageReport {
    /// Provides the amount of storage a user has remaining on their authorization at the specific
    /// storage host. If the user has managed to go over their quota somehow this will return 0. It
    /// will never return a negative number.
    pub fn authorization_available(&self) -> i64 {
        (self.maximum_authorized - self.current_consumption).max(0)
    }

    pub fn current_consumption(&self) -> i64 {
        self.current_consumption
    }

    pub fn maximum_authorized(&self) -> i64 {
        self.maximum_authorized
    }
}
