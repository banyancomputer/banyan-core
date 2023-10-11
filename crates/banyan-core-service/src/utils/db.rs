use crate::db::models::{self, BucketType, CreatedResource, StorageClass};
use crate::email::message::EmailMessage;
use crate::extractors::DbConn;
use crate::utils::keys::fingerprint_public_pem;

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

pub async fn select_storage_host(db_conn: &mut DbConn) -> Result<models::StorageHost, sqlx::Error> {
    sqlx::query_as!(
        models::StorageHost,
        r#"SELECT id, name, url, used_storage, available_storage, fingerprint, pem FROM storage_hosts ORDER BY RANDOM() LIMIT 1;"#,
    )
    .fetch_one(&mut *db_conn.0)
    .await
}

pub async fn record_storage_grant(
    storage_host_id: &str,
    account_id: &str,
    metadata_id: &str,
    authorized_usage: u64,
    db_conn: &mut DbConn,
) -> Result<String, sqlx::Error> {
    let authorized_usage = authorized_usage as i64;

    let storage_grant_id: String = sqlx::query_scalar!(
        r#"
            INSERT INTO
                storage_grants (storage_host_id, account_id, authorized_amount)
                VALUES ($1, $2, $3)
                RETURNING id;"#,
        storage_host_id,
        account_id,
        authorized_usage,
    )
    .fetch_one(&mut *db_conn.0)
    .await?;

    sqlx::query!(r#"
            INSERT INTO
                storage_hosts_metadatas_storage_grants (storage_host_id, metadata_id, storage_grant_id)
                VALUES ($1, $2, $3);"#,
            storage_host_id,
            metadata_id,
            storage_grant_id,
        )
        .execute(&mut *db_conn.0)
        .await?;

    Ok(storage_grant_id)
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
) -> Result<(), sqlx::Error> {
    // delete does not tell us whether any rows existed, to return a 404 we need to see if its
    // present or not. We'll cheat and use our read bucket method for this 404 check.
    read_bucket(account_id, bucket_id, db_conn).await?;

    sqlx::query!(
        r#"DELETE FROM buckets WHERE id = $1 AND account_id = $2"#,
        bucket_id,
        account_id,
    )
    .execute(&mut *db_conn.0)
    .await?;

    Ok(())
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
    approved: bool,
    pem: &str,
    db_conn: &mut DbConn,
) -> Result<CreatedResource, sqlx::Error> {
    let fingerprint = fingerprint_public_pem(pem);
    sqlx::query_as!(
        models::CreatedResource,
        r#"INSERT INTO bucket_keys (bucket_id, approved, pem, fingerprint) VALUES ($1, $2, $3, $4) RETURNING id;"#,
        bucket_id,
        approved,
        pem,
        fingerprint
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
        r#"SELECT id, bucket_id, approved, pem, fingerprint FROM bucket_keys WHERE id = $1 AND bucket_id = $2;"#,
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
        r#"SELECT id, bucket_id, approved, pem, fingerprint FROM bucket_keys WHERE bucket_id = $1;"#,
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
        r#"DELETE FROM bucket_keys WHERE id = $1 AND bucket_id = $2 RETURNING id, bucket_id, approved, pem, fingerprint;"#,
        bucket_key_id,
        bucket_id,
    )
    .fetch_one(&mut *db_conn.0)
    .await
}

/// Approve a bucket key for use by its id and authorize that it belongs to a given bucket_id.
/// # Arguments
/// * `bucket_id` - The id of the bucket to read.
/// * `pem` - The public PEM of the Key
/// * `db_conn` - The database connection to use.
/// # Return Type
/// Returns the bucket key if it exists and belongs to the given bucket_id, otherwise returns an error.
pub async fn approve_bucket_key(
    bucket_id: &str,
    fingerprint: &str,
    db_conn: &mut DbConn,
) -> Result<models::BucketKey, sqlx::Error> {
    // Perorm the update
    sqlx::query_as!(
        models::BucketKey,
        r#"
        UPDATE bucket_keys SET 
        approved = true 
        WHERE fingerprint = $1 AND bucket_id = $2 
        RETURNING id, bucket_id, approved, pem, fingerprint;"#,
        fingerprint,
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
) -> Result<models::MetadataWithSnapshot, sqlx::Error> {
    let maybe_metadata = sqlx::query_as!(
        models::Metadata,
        r#"SELECT id, bucket_id, root_cid, metadata_cid, expected_data_size, data_size as "data_size!", state, metadata_size as "metadata_size!", metadata_hash as "metadata_hash!", created_at, updated_at
        FROM metadata WHERE id = $1 AND bucket_id = $2;"#,
        metadata_id,
        bucket_id,
    )
    .fetch_one(&mut *db_conn.0)
    .await;
    let metadata = match maybe_metadata {
        Ok(metadata) => metadata,
        Err(err) => match err {
            sqlx::Error::RowNotFound => return Err(sqlx::Error::RowNotFound),
            _ => return Err(err),
        },
    };
    let maybe_snapshot_id = sqlx::query_as!(
        models::CreatedResource,
        r#"SELECT 
            s.id as "id"
        FROM 
            snapshots s
        INNER JOIN 
            metadata m ON m.id = s.metadata_id
        WHERE 
            m.id = $1 AND m.bucket_id = $2;"#,
        metadata_id,
        bucket_id
    )
    .fetch_optional(&mut *db_conn.0)
    .await?;
    let metadata_with_snapshot = match maybe_snapshot_id {
        Some(cr) => models::MetadataWithSnapshot {
            metadata,
            snapshot_id: Some(cr.id),
        },
        None => models::MetadataWithSnapshot {
            metadata,
            snapshot_id: None,
        },
    };
    Ok(metadata_with_snapshot)
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
) -> Result<models::MetadataWithSnapshot, sqlx::Error> {
    let maybe_metadata = sqlx::query_as!(
        models::Metadata,
        r#"SELECT id, bucket_id, root_cid, metadata_cid, expected_data_size, data_size as "data_size!", state, metadata_size as "metadata_size!", metadata_hash as "metadata_hash!", created_at, updated_at
        FROM metadata WHERE bucket_id = $1 AND state = 'current';"#,
        bucket_id,
    )
    .fetch_one(&mut *db_conn.0)
    .await;
    let metadata = match maybe_metadata {
        Ok(metadata) => metadata,
        Err(err) => match err {
            sqlx::Error::RowNotFound => return Err(sqlx::Error::RowNotFound),
            _ => return Err(err),
        },
    };
    let maybe_snapshot_id = sqlx::query_as!(
        models::CreatedResource,
        r#"SELECT 
            s.id as "id"
        FROM 
            snapshots s
        INNER JOIN 
            metadata m ON m.id = s.metadata_id
        WHERE 
            m.id = $1 AND m.bucket_id = $2;"#,
        metadata.id,
        bucket_id
    )
    .fetch_optional(&mut *db_conn.0)
    .await?;
    let metadata_with_snapshot = match maybe_snapshot_id {
        Some(cr) => models::MetadataWithSnapshot {
            metadata,
            snapshot_id: Some(cr.id),
        },
        None => models::MetadataWithSnapshot {
            metadata,
            snapshot_id: None,
        },
    };
    Ok(metadata_with_snapshot)
}

pub async fn create_snapshot(
    metadata_id: &str,
    db_conn: &mut DbConn,
) -> Result<String, sqlx::Error> {
    let metadata_size: i64 = sqlx::query_scalar(
        r#"
            SELECT metadata_size + COALESCE(expected_data_size, data_size)
            FROM metadata
            WHERE id = $1;"#,
    )
    .bind(metadata_id)
    .fetch_one(&mut *db_conn.0)
    .await?;

    sqlx::query_scalar::<sqlx::Sqlite, String>(
        r#"INSERT INTO snapshots (metadata_id, size)
        VALUES ($1, $2)
        RETURNING id;"#,
    )
    .bind(metadata_id)
    .bind(metadata_size)
    .fetch_one(&mut *db_conn.0)
    .await
}

/// Returns information about a specific snapshot. The caller must know what bucket its associated
/// with as an authorization check.
pub async fn read_snapshot(
    bucket_id: &str,
    snapshot_id: &str,
    db_conn: &mut DbConn,
) -> Result<models::Snapshot, sqlx::Error> {
    sqlx::query_as!(
        models::Snapshot,
        r#"SELECT s.id, s.metadata_id, s.size as "size!", s.created_at
             FROM snapshots AS s
             INNER JOIN metadata m ON m.id = s.metadata_id
             WHERE s.id = $1 AND m.bucket_id = $2;"#,
        snapshot_id,
        bucket_id
    )
    .fetch_one(&mut *db_conn.0)
    .await
}

/// Returns all snapshots associated with a specific bucket
pub async fn read_all_snapshots(
    bucket_id: &str,
    db_conn: &mut DbConn,
) -> Result<Vec<models::Snapshot>, sqlx::Error> {
    sqlx::query_as!(
        models::Snapshot,
        r#"SELECT s.id, s.metadata_id, s.size as "size!", s.created_at
                FROM snapshots AS s
                INNER JOIN metadata m ON m.id = s.metadata_id
                WHERE m.bucket_id = $1;"#,
        bucket_id
    )
    .fetch_all(&mut *db_conn.0)
    .await
}

/// Read the total data storage consumed by both data and metadata across a user's entire account
pub async fn read_total_usage(account_id: &str, db_conn: &mut DbConn) -> Result<u64, sqlx::Error> {
    sqlx::query_scalar::<sqlx::Sqlite, i64>(
        r#"SELECT
             SUM(m.metadata_size + COALESCE(m.expected_data_size, m.data_size))
           FROM metadata as m
           INNER JOIN buckets b ON b.id = m.bucket_id
           WHERE b.account_id = $1;"#,
    )
    .bind(account_id)
    .fetch_one(&mut *db_conn.0)
    .await
    .map(|num| num as u64)
}

/// Read just the data usage of the given account id across all buckets they control
pub async fn read_total_data_usage(
    account_id: &str,
    db_conn: &mut DbConn,
) -> Result<u64, sqlx::Error> {
    sqlx::query_scalar::<sqlx::Sqlite, i64>(
        r#"SELECT
                SUM(COALESCE(COALESCE(m.data_size, m.expected_data_size), 0)) + 
                SUM(COALESCE(m.metadata_size, 0))
            FROM
                metadata m
            INNER JOIN
                buckets b ON b.id = m.bucket_id
            WHERE
                b.account_id = $1 AND m.state IN ('current', 'outdated', 'pending');"#,
    )
    .bind(account_id)
    .fetch_one(&mut *db_conn.0)
    .await
    .map(|num| num as u64)
}

/// Read the data usage of a given bucket id.
pub async fn read_bucket_data_usage(
    bucket_id: &str,
    db_conn: &mut DbConn,
) -> Result<u64, sqlx::Error> {
    sqlx::query_scalar::<sqlx::Sqlite, i64>(
        r#"SELECT
             SUM(m.metadata_size + COALESCE(m.expected_data_size, m.data_size))
           FROM metadata as m
           WHERE bucket_id = $1;"#,
    )
    .bind(bucket_id)
    .fetch_one(&mut *db_conn.0)
    .await
    .map(|num| num as u64)
}

#[allow(dead_code)]
pub async fn record_sent_email(
    account_id: &str,
    email_message: &impl EmailMessage,
    db_conn: &mut DbConn,
) -> Result<(), sqlx::Error> {
    let type_name = email_message.type_name();
    sqlx::query!(
        r#"INSERT INTO emails (account_id, type) VALUES ($1, $2);"#,
        account_id,
        type_name
    )
    .execute(&mut *db_conn.0)
    .await?;
    Ok(())
}
