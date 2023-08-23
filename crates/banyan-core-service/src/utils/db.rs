use crate::db::models;
use crate::extractors::DbConn;

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
    let maybe_bucket = sqlx::query_as!(
        models::Bucket,
        r#"SELECT id, account_id, name, type, storage_class FROM buckets WHERE id = $1 AND account_id = $2"#,
        bucket_id,
        account_id,
    )
    .fetch_one(&mut *db_conn.0)
    .await;
    match maybe_bucket {
        Ok(bucket) => Ok(bucket),
        Err(err) => match err {
            sqlx::Error::RowNotFound => Err(sqlx::Error::RowNotFound),
            _ => Err(err),
        },
    }
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
    let maybe_buckets = sqlx::query_as!(
        models::Bucket,
        r#"SELECT id, account_id, name, type, storage_class FROM buckets WHERE account_id = $1"#,
        account_id,
    )
    .fetch_all(&mut *db_conn.0)
    .await;
    match maybe_buckets {
        Ok(buckets) => Ok(buckets),
        Err(err) => match err {
            sqlx::Error::RowNotFound => Err(sqlx::Error::RowNotFound),
            _ => Err(err),
        },
    }
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
    let maybe_bucket = sqlx::query_as!(
        models::Bucket,
        r#"DELETE FROM buckets WHERE id = $1 AND account_id = $2 RETURNING id, account_id, name, type, storage_class"#,
        bucket_id,
        account_id,
    )
    .fetch_one(&mut *db_conn.0)
    .await;
    match maybe_bucket {
        Ok(bucket) => Ok(bucket),
        Err(err) => Err(err),
    }
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
    let maybe_bucket = sqlx::query_as!(
        models::CreatedResource,
        r#"SELECT id FROM buckets WHERE id = $1 AND account_id = $2;"#,
        bucket_id,
        account_id
    )
    .fetch_one(&mut *db_conn.0)
    .await;
    match maybe_bucket {
        Ok(_) => Ok(()),
        Err(err) => match err {
            sqlx::Error::RowNotFound => Err(sqlx::Error::RowNotFound),
            _ => Err(err),
        },
    }
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
    let maybe_bucket_key = sqlx::query_as!(
        models::BucketKey,
        r#"SELECT id, bucket_id, approved, pem FROM bucket_keys WHERE id = $1 AND bucket_id = $2;"#,
        bucket_key_id,
        bucket_id,
    )
    .fetch_one(&mut *db_conn.0)
    .await;
    match maybe_bucket_key {
        Ok(bucket_key) => Ok(bucket_key),
        Err(err) => match err {
            sqlx::Error::RowNotFound => Err(sqlx::Error::RowNotFound),
            _ => Err(err),
        },
    }
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
    let maybe_bucket_keys = sqlx::query_as!(
        models::BucketKey,
        r#"SELECT id, bucket_id, approved, pem FROM bucket_keys WHERE bucket_id = $1;"#,
        bucket_id,
    )
    .fetch_all(&mut *db_conn.0)
    .await;
    match maybe_bucket_keys {
        Ok(bucket_keys) => Ok(bucket_keys),
        Err(err) => match err {
            sqlx::Error::RowNotFound => Err(sqlx::Error::RowNotFound),
            _ => Err(err),
        },
    }
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
    let maybe_bucket_key = sqlx::query_as!(
        models::BucketKey,
        r#"DELETE FROM bucket_keys WHERE id = $1 AND bucket_id = $2 RETURNING id, bucket_id, approved, pem;"#,
        bucket_key_id,
        bucket_id,
    )
    .fetch_one(&mut *db_conn.0)
    .await;
    match maybe_bucket_key {
        Ok(bucket_key) => Ok(bucket_key),
        Err(err) => match err {
            sqlx::Error::RowNotFound => Err(sqlx::Error::RowNotFound),
            _ => Err(err),
        },
    }
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
    let maybe_metadata = sqlx::query_as!(
        models::Metadata,
        r#"SELECT id, bucket_id, root_cid, metadata_cid, data_size, state, metadata_size as "metadata_size!", metadata_hash as "metadata_hash!", created_at, updated_at
        FROM metadata WHERE id = $1 AND bucket_id = $2;"#,
        metadata_id,
        bucket_id,
    )
    .fetch_one(&mut *db_conn.0)
    .await;
    match maybe_metadata {
        Ok(metadata) => Ok(metadata),
        Err(err) => match err {
            sqlx::Error::RowNotFound => Err(sqlx::Error::RowNotFound),
            _ => Err(err),
        },
    }
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
    let maybe_metadata = sqlx::query_as!(
        models::CreatedResource,
        r#"SELECT id FROM metadata WHERE id = $1 AND bucket_id = $2;"#,
        metadata_id,
        bucket_id,
    )
    .fetch_one(&mut *db_conn.0)
    .await;
    match maybe_metadata {
        Ok(_) => Ok(()),
        Err(err) => match err {
            sqlx::Error::RowNotFound => Err(sqlx::Error::RowNotFound),
            _ => Err(err),
        },
    }
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
    let maybe_metadata = sqlx::query_as!(
        models::Metadata,
        r#"SELECT id, bucket_id, root_cid, metadata_cid, data_size, state, metadata_size as "metadata_size!", metadata_hash as "metadata_hash!", created_at, updated_at
        FROM metadata WHERE bucket_id = $1;"#,
        bucket_id,
    )
    .fetch_all(&mut *db_conn.0)
    .await;
    match maybe_metadata {
        Ok(metadata) => Ok(metadata),
        Err(err) => match err {
            sqlx::Error::RowNotFound => Err(sqlx::Error::RowNotFound),
            _ => Err(err),
        },
    }
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
    let maybe_metadata = sqlx::query_as!(
        models::Metadata,
        r#"SELECT id, bucket_id, root_cid, metadata_cid, data_size, state, metadata_size as "metadata_size!", metadata_hash as "metadata_hash!", created_at, updated_at
        FROM metadata WHERE bucket_id = $1 AND state = 'current';"#,
        bucket_id,
    )
    .fetch_one(&mut *db_conn.0)
    .await;
    match maybe_metadata {
        Ok(metadata) => Ok(metadata),
        Err(err) => match err {
            sqlx::Error::RowNotFound => Err(sqlx::Error::RowNotFound),
            _ => Err(err),
        },
    }
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
    let maybe_snapshot = sqlx::query_as!(
        models::Snapshot,
        r#"INSERT INTO snapshots (metadata_id)
        VALUES ($1)
        RETURNING id, metadata_id, created_at;"#,
        metadata_id
    )
    .fetch_one(&mut *db_conn.0)
    .await;
    match maybe_snapshot {
        Ok(snapshot) => Ok(snapshot),
        Err(err) => Err(err),
    }
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
    let maybe_snapshot = sqlx::query_as!(
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
    .await;
    match maybe_snapshot {
        Ok(snapshot) => Ok(snapshot),
        Err(err) => match err {
            sqlx::Error::RowNotFound => Err(sqlx::Error::RowNotFound),
            _ => Err(err),
        },
    }
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
    let maybe_snapshots = sqlx::query_as!(
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
    .await;
    match maybe_snapshots {
        Ok(snapshots) => Ok(snapshots),
        Err(err) => match err {
            sqlx::Error::RowNotFound => Err(sqlx::Error::RowNotFound),
            _ => Err(err),
        },
    }
}
