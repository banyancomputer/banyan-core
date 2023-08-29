use crate::db::models::{self, BucketType, CreatedResource, StorageClass};
use crate::extractors::DbConn;
use axum::response::{IntoResponse, Response};
use http::StatusCode;
use serde::Serialize;

/// Process an SQLX error in a reusable format for responding with error messages
pub fn sqlx_error_to_response(err: sqlx::Error, operation: &str, resource: &str) -> Response {
    let default = (
        StatusCode::INTERNAL_SERVER_ERROR,
        "internal server error".to_string(),
    )
        .into_response();

    match err {
        sqlx::Error::Database(db_err) => {
            if db_err.is_unique_violation() {
                (
                    StatusCode::CONFLICT,
                    format!("{} with that name already exists", resource),
                )
                    .into_response()
            } else {
                tracing::error!("unable to {} {}: {}", operation, resource, db_err);
                default
            }
        }
        sqlx::Error::RowNotFound => (
            StatusCode::NOT_FOUND,
            format!("{} not found: {}", resource, err),
        )
            .into_response(),
        // Catch all others
        _ => {
            tracing::error!("unable to {} {}: {}", operation, resource, err);
            default
        }
    }
}

/// Create a new Bucket in the database and return the created resource.
/// Implements an authorized read of a bucket by id and account_id.
/// # Arguments
/// * `account_id` - The account_id of the account that owns the bucket.
/// * `type` - The type of the bucket.
/// * `storage_class` - The storage class of the bucket.
/// * `db_conn` - The database connection to use.
/// # Return Type
/// Returns the created resource if successful, otherwise errors.
pub async fn create_bucket(
    account_id: &str,
    name: &str,
    r#type: &BucketType,
    storage_class: &StorageClass,
    db_conn: &mut DbConn,
) -> Result<CreatedResource, sqlx::Error> {
    sqlx::query_as!(
        models::CreatedResource,
        r#"INSERT INTO buckets (account_id, name, type, storage_class) VALUES ($1, $2, $3, $4) RETURNING id;"#,
        account_id,
        name,
        r#type,
        storage_class,
    )
    .fetch_one(&mut *db_conn.0)
    .await
}

/// Pull the bucket from the database by id and account_id and return it.
/// Implements an authorized read of a bucket by id and account_id.
/// # Arguments
/// * `account_id` - The account_id of the account that owns the bucket.
/// * `bucket_id` - The id of the bucket to read.
/// * `db_conn` - The database connection to use.
/// # Return Type
/// Returns the bucket if it exists and is owned by the given account_id, otherwise returns an error.
pub async fn read_bucket(
    account_id: &str,
    bucket_id: &str,
    db_conn: &mut DbConn,
) -> Result<models::Bucket, sqlx::Error> {
    sqlx::query_as!(
        models::Bucket,
        r#"SELECT id, account_id, name, type, storage_class FROM buckets WHERE id = $1 AND account_id = $2"#,
        bucket_id,
        account_id,
    )
    .fetch_one(&mut *db_conn.0)
    .await
}

/// Pull all buckets from the database by account_id and return them.
/// Implements an authorized read of all buckets by account_id.
/// # Arguments
/// * `account_id` - The account_id of the account that owns the buckets.
/// * `db_conn` - The database connection to use.
/// # Return Type
/// Returns a vector of buckets if they exist and are owned by the given account_id, otherwise returns an error.
pub async fn read_all_buckets(
    account_id: &str,
    db_conn: &mut DbConn,
) -> Result<Vec<models::Bucket>, sqlx::Error> {
    sqlx::query_as!(
        models::Bucket,
        r#"SELECT id, account_id, name, type, storage_class FROM buckets WHERE account_id = $1"#,
        account_id,
    )
    .fetch_all(&mut *db_conn.0)
    .await
}

/// Delete a bucket by id and account_id and return it.
/// Implements an authorized delete of a bucket by id and account_id.
/// # Arguments
/// * `account_id` - The account_id of the account that owns the bucket.
/// * `bucket_id` - The id of the bucket to delete.
/// * `db_conn` - The database connection to use.
/// # Return Type
/// Returns the bucket if it exists and is owned by the given account_id, otherwise returns an error.
pub async fn delete_bucket(
    account_id: &str,
    bucket_id: &str,
    db_conn: &mut DbConn,
) -> Result<models::Bucket, sqlx::Error> {
    sqlx::query_as!(
        models::Bucket,
        r#"DELETE FROM buckets WHERE id = $1 AND account_id = $2 RETURNING id, account_id, name, type, storage_class"#,
        bucket_id,
        account_id,
    )
    .fetch_one(&mut *db_conn.0)
    .await
}

/// Authorize the given account_id to read the given bucket_id.
/// # Arguments
/// * `account_id` - The account_id of the account that owns the bucket.
/// * `bucket_id` - The id of the bucket to read.
/// * `db_conn` - The database connection to use.
/// # Return Type
/// Returns `Ok(())` if the account_id is authorized to read the bucket_id, otherwise returns an error.
pub async fn authorize_bucket(
    account_id: &str,
    bucket_id: &str,
    db_conn: &mut DbConn,
) -> Result<(), sqlx::Error> {
    sqlx::query_as!(
        models::CreatedResource,
        r#"SELECT id FROM buckets WHERE id = $1 AND account_id = $2;"#,
        bucket_id,
        account_id
    )
    .fetch_one(&mut *db_conn.0)
    .await
    .map(|_| ())
}

/// Create a bucket key by its id and PEM
/// # Arguments
/// * `bucket_id` - The id of the bucket to insert this key in.
/// * `pem` - The public PEM of the Key
/// * `db_conn` - The database connection to use.
/// # Return Type
/// Returns the created resource if creation succeeds, otherwise returns an error.
pub async fn create_bucket_key(
    bucket_id: &str,
    pem: &str,
    db_conn: &mut DbConn,
) -> Result<CreatedResource, sqlx::Error> {
    sqlx::query_as!(
        models::CreatedResource,
        r#"INSERT INTO bucket_keys (bucket_id, approved, pem) VALUES ($1, false, $2) RETURNING id;"#,
        bucket_id,
        pem,
    )
    .fetch_one(&mut *db_conn.0)
    .await
}

/// Read a bucket key by its id and authorize that it belongs to a given bucket_id.
/// # Arguments
/// * `bucket_id` - The id of the bucket to read.
/// * `bucket_key_id` - The id of the bucket key to read.
/// * `db_conn` - The database connection to use.
/// # Return Type
/// Returns the bucket key if it exists and belongs to the given bucket_id, otherwise returns an error.
pub async fn read_bucket_key(
    bucket_id: &str,
    bucket_key_id: &str,
    db_conn: &mut DbConn,
) -> Result<models::BucketKey, sqlx::Error> {
    sqlx::query_as!(
        models::BucketKey,
        r#"SELECT id, bucket_id, approved, pem FROM bucket_keys WHERE id = $1 AND bucket_id = $2;"#,
        bucket_key_id,
        bucket_id,
    )
    .fetch_one(&mut *db_conn.0)
    .await
}

/// Read all bucket keys by a given bucket_id.
/// # Arguments
/// * `bucket_id` - The id of the bucket to read.
/// * `db_conn` - The database connection to use.
/// # Return Type
/// Returns a vector of bucket keys if they exist and belong to the given bucket_id, otherwise returns an error.
pub async fn read_all_bucket_keys(
    bucket_id: &str,
    db_conn: &mut DbConn,
) -> Result<Vec<models::BucketKey>, sqlx::Error> {
    sqlx::query_as!(
        models::BucketKey,
        r#"SELECT id, bucket_id, approved, pem FROM bucket_keys WHERE bucket_id = $1;"#,
        bucket_id,
    )
    .fetch_all(&mut *db_conn.0)
    .await
}

/// Delete a bucket key by its id and authorize that it belongs to a given bucket_id.
/// # Arguments
/// * `bucket_id` - The id of the bucket to read.
/// * `bucket_key_id` - The id of the bucket key to read.
/// * `db_conn` - The database connection to use.
/// # Return Type
/// Returns the bucket key if it exists and belongs to the given bucket_id, otherwise returns an error.
pub async fn delete_bucket_key(
    bucket_id: &str,
    bucket_key_id: &str,
    db_conn: &mut DbConn,
) -> Result<models::BucketKey, sqlx::Error> {
    sqlx::query_as!(
        models::BucketKey,
        r#"DELETE FROM bucket_keys WHERE id = $1 AND bucket_id = $2 RETURNING id, bucket_id, approved, pem;"#,
        bucket_key_id,
        bucket_id,
    )
    .fetch_one(&mut *db_conn.0)
    .await
}

/// Approve a bucket key for use by its id and authorize that it belongs to a given bucket_id.
/// # Arguments
/// * `bucket_id` - The id of the bucket to read.
/// * `bucket_key_id` - The id of the bucket key to read.
/// * `db_conn` - The database connection to use.
/// # Return Type
/// Returns the bucket key if it exists and belongs to the given bucket_id, otherwise returns an error.
pub async fn approve_bucket_key(
    bucket_id: &str,
    bucket_key_id: &str,
    db_conn: &mut DbConn,
) -> Result<models::BucketKey, sqlx::Error> {
    // Perorm the update
    sqlx::query_as!(
        models::BucketKey,
        r#"
        UPDATE bucket_keys SET 
        approved = true 
        WHERE id = $1 AND bucket_id = $2 
        RETURNING id, bucket_id, approved, pem;"#,
        bucket_key_id,
        bucket_id,
    )
    .fetch_one(&mut *db_conn.0)
    .await
}

/// Read metadata from the database, checking if references a given bucket_id.
/// # Arguments
/// * `bucket_id` - The id of the bucket to read.
/// * `metadata_id` - The id of the metadata to read.
/// * `db_conn` - The database connection to use.
/// # Return Type
/// Returns the metadata if it exists and belongs to the given bucket_id, otherwise returns an error.
pub async fn read_metadata(
    bucket_id: &str,
    metadata_id: &str,
    db_conn: &mut DbConn,
) -> Result<models::Metadata, sqlx::Error> {
    sqlx::query_as!(
        models::Metadata,
        r#"SELECT id, bucket_id, root_cid, metadata_cid, expected_data_size, data_size as "data_size!", state, metadata_size as "metadata_size!", metadata_hash as "metadata_hash!", created_at, updated_at
        FROM metadata WHERE id = $1 AND bucket_id = $2;"#,
        metadata_id,
        bucket_id,
    )
    .fetch_one(&mut *db_conn.0)
    .await
}

/// Authorize access to the given metadata_id by checking if it references a given bucket_id.
/// # Arguments
/// * `bucket_id` - The id of the bucket to read.
/// * `metadata_id` - The id of the metadata to read.
/// * `db_conn` - The database connection to use.
/// # Return Type
/// Returns `Ok(())` if the metadata_id references the given bucket_id, otherwise returns an error.
pub async fn authorize_metadata(
    bucket_id: &str,
    metadata_id: &str,
    db_conn: &mut DbConn,
) -> Result<(), sqlx::Error> {
    sqlx::query_as!(
        models::CreatedResource,
        r#"SELECT id FROM metadata WHERE id = $1 AND bucket_id = $2;"#,
        metadata_id,
        bucket_id,
    )
    .fetch_one(&mut *db_conn.0)
    .await
    .map(|_| ())
}

/// Read all metadata from the database by a given bucket_id.
/// # Arguments
/// * `bucket_id` - The id of the bucket to read.
/// * `db_conn` - The database connection to use.
/// # Return Type
/// Returns a vector of metadata if it exists and belongs to the given bucket_id, otherwise returns an error.
pub async fn read_all_metadata(
    bucket_id: &str,
    db_conn: &mut DbConn,
) -> Result<Vec<models::Metadata>, sqlx::Error> {
    sqlx::query_as!(
        models::Metadata,
        r#"SELECT id, bucket_id, root_cid, metadata_cid, expected_data_size, data_size as "data_size!", state, metadata_size as "metadata_size!", metadata_hash as "metadata_hash!", created_at, updated_at
        FROM metadata WHERE bucket_id = $1;"#,
        bucket_id,
    )
    .fetch_all(&mut *db_conn.0)
    .await
}

/// Read the current metadata from the database by a given bucket_id.
/// # Arguments
/// * `bucket_id` - The id of the bucket to read.
/// * `db_conn` - The database connection to use.
/// # Return Type
/// Returns the current metadata if it exists and belongs to the given bucket_id, otherwise returns an error.
pub async fn read_current_metadata(
    bucket_id: &str,
    db_conn: &mut DbConn,
) -> Result<models::Metadata, sqlx::Error> {
    sqlx::query_as!(
        models::Metadata,
        r#"SELECT id, bucket_id, root_cid, metadata_cid, expected_data_size, data_size as "data_size!", state, metadata_size as "metadata_size!", metadata_hash as "metadata_hash!", created_at, updated_at
        FROM metadata WHERE bucket_id = $1 AND state = 'current';"#,
        bucket_id,
    )
    .fetch_one(&mut *db_conn.0)
    .await
}

/// Create a snapshot and return the created resource.
/// # Arguments
/// * `metadata_id` - The id of the metadata to snapshot.
/// * `db_conn` - The database connection to use.
/// # Return Type
/// Returns the created snapshot if successful, otherwise returns an error.
pub async fn create_snapshot(
    metadata_id: &str,
    db_conn: &mut DbConn,
) -> Result<models::Snapshot, sqlx::Error> {
    sqlx::query_as!(
        models::Snapshot,
        r#"INSERT INTO snapshots (metadata_id)
        VALUES ($1)
        RETURNING id, metadata_id, created_at;"#,
        metadata_id
    )
    .fetch_one(&mut *db_conn.0)
    .await
}

/// Read a snapshot by its id and authorize that its associated metadata belongs to a given bucket_id.
/// # Arguments
/// * `bucket_id` - The id of the bucket to read.
/// * `snapshot_id` - The id of the snapshot to read.
/// * `db_conn` - The database connection to use.
/// # Return Type
/// Returns the snapshot if it exists and belongs to the given bucket_id, otherwise returns an error.
pub async fn read_snapshot(
    bucket_id: &str,
    snapshot_id: &str,
    db_conn: &mut DbConn,
) -> Result<models::Snapshot, sqlx::Error> {
    sqlx::query_as!(
        models::Snapshot,
        r#"SELECT 
            s.id,
            s.metadata_id as "metadata_id!",
            s.created_at as "created_at!"
        FROM 
            snapshots s
        INNER JOIN 
            metadata m ON m.id = s.metadata_id
        WHERE 
            s.id = $1 AND m.bucket_id = $2;"#,
        snapshot_id,
        bucket_id
    )
    .fetch_one(&mut *db_conn.0)
    .await
}

/// Read a snapshot by bucket_id and snapshot_id.
/// # Arguments
/// * `bucket_id` - The id of the bucket to read.
/// * `snapshot_id` - The id of the snapshot to read.
/// * `db_conn` - The database connection to use.
/// # Return Type
/// Returns the snapshot if it exists and belongs to the given bucket_id, otherwise returns an error.
pub async fn read_all_snapshots(
    bucket_id: &str,
    db_conn: &mut DbConn,
) -> Result<Vec<models::Snapshot>, sqlx::Error> {
    sqlx::query_as!(
        models::Snapshot,
        r#"SELECT 
            s.id,
            s.metadata_id as "metadata_id!",
            s.created_at as "created_at!"
        FROM 
            snapshots s
        INNER JOIN 
            metadata m ON m.id = s.metadata_id
        WHERE 
            m.bucket_id = $1;"#,
        bucket_id
    )
    .fetch_all(&mut *db_conn.0)
    .await
}

/// Read storage host by name.
/// # Arguments
/// * `name` - The name of the storage host to read.
/// * `db_conn` - The database connection to use.
/// # Return Type
/// Returns the storage host if it exists, otherwise returns an error.
pub async fn read_storage_host(
    name: &str,
    db_conn: &mut DbConn,
) -> Result<models::StorageHost, sqlx::Error> {
    sqlx::query_as!(
        models::StorageHost,
        r#"SELECT id, name, url, used_storage, available_storage, fingerprint, pem FROM storage_hosts WHERE name = $1;"#,
        name,
    )
    .fetch_one(&mut *db_conn.0)
    .await
}

/// Read the data + metadata usage of the given account id.
/// This is the sum of all data_size and metadata_size for all metadata associated with the account.
/// # Arguments
/// * `account_id` - The id of the account to read.
/// * `metadata_states` - The states of the metadata to include in the usage calculation.
/// * `bucket_ids` - The ids of the buckets to include in the usage calculation. If empty, all buckets are included.
/// * `db_conn` - The database connection to use.
/// # Return Type
/// Returns the current data usage if it exists, otherwise returns an error.
pub async fn read_usage(
    account_id: &str,
    metadata_states: Vec<models::MetadataState>,
    bucket_ids: Option<Vec<String>>,
    db_conn: &mut DbConn,
) -> Result<u64, sqlx::Error> {
    let states = format!(
        "\'{}\'",
        metadata_states
            .iter()
            .map(|state| state.to_string())
            .collect::<Vec<String>>()
            .join("', '")
    );
    match bucket_ids {
        Some(bucket_ids) => {
            let bucket_ids = format!("\'{}\'", bucket_ids.join("', '"));
            sqlx::query_as!(
                GetTotalUsage,
                r#"SELECT 
                    COALESCE(SUM(COALESCE(m.data_size, m.expected_data_size)), 0) as "data_size!",
                    COALESCE(SUM(m.metadata_size), 0) as "metadata_size!"
                FROM
                    metadata m
                INNER JOIN
                    buckets b ON b.id = m.bucket_id
                WHERE
                    b.account_id = $1 AND m.state IN ($2) AND b.id IN ($3);"#,
                account_id,
                states,
                bucket_ids
            )
            .fetch_one(&mut *db_conn.0)
            .await
        }
        None => {
            sqlx::query_as!(
                GetTotalUsage,
                r#"SELECT 
                COALESCE(SUM(m.data_size), 0) as "data_size!",
                COALESCE(SUM(m.metadata_size), 0) as "metadata_size!"
            FROM
                metadata m
            INNER JOIN
                buckets b ON b.id = m.bucket_id
            WHERE
                b.account_id = $1 AND m.state IN ($2);"#,
                account_id,
                states
            )
            .fetch_one(&mut *db_conn.0)
            .await
        }
    }
    .map(|usage| (usage.data_size + usage.metadata_size) as u64)
}

/// Read the data usage of the given account id.
/// This is the sum of all data_size for all metadata associated with the account.
/// # Arguments
/// * `account_id` - The id of the account to read.
/// * `metadata_states` - The states of the metadata to include in the usage calculation.
/// * `bucket_ids` - The ids of the buckets to include in the usage calculation. If empty, all buckets are included.
/// * `db_conn` - The database connection to use.
/// # Return Type
/// Returns the data usage if it exists, otherwise returns an error.
pub async fn read_data_usage(
    account_id: &str,
    metadata_states: Vec<models::MetadataState>,
    bucket_ids: Option<Vec<String>>,
    db_conn: &mut DbConn,
) -> Result<u64, sqlx::Error> {
    let states = format!(
        "\'{}\'",
        metadata_states
            .iter()
            .map(|state| state.to_string())
            .collect::<Vec<String>>()
            .join("', '")
    );

    match bucket_ids {
        Some(bucket_ids) => {
            let bucket_ids = format!("\'{}\'", bucket_ids.join("', '"));
            sqlx::query_as!(
                GetUsage,
                r#"SELECT 
                    COALESCE(SUM(COALESCE(m.data_size, m.expected_data_size)), 0) as "size!"
                FROM
                    metadata m
                INNER JOIN
                    buckets b ON b.id = m.bucket_id
                WHERE
                    b.account_id = $1 AND m.state IN ($2) AND b.id IN ($3);"#,
                account_id,
                states,
                bucket_ids
            )
            .fetch_one(&mut *db_conn.0)
            .await
        }
        None => {
            sqlx::query_as!(
                GetUsage,
                r#"SELECT 
                COALESCE(SUM(m.data_size), 0) as "size!"
            FROM
                metadata m
            INNER JOIN
                buckets b ON b.id = m.bucket_id
            WHERE
                b.account_id = $1 AND m.state IN ($2);"#,
                account_id,
                states
            )
            .fetch_one(&mut *db_conn.0)
            .await
        }
    }
    .map(|usage| usage.size as u64)
}

#[derive(Serialize)]
struct GetTotalUsage {
    pub data_size: i64,
    pub metadata_size: i64,
}

#[derive(Serialize)]
struct GetUsage {
    pub size: i64,
}
