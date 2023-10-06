use crate::db::models::{self, BucketType, CreatedResource, StorageClass};
use crate::database::Database;
use crate::utils::keys::fingerprint_public_pem;
use serde::Serialize;
use sqlx::FromRow;

/// Create a new Bucket in the database and return the created resource. Implements an authorized
/// read of a bucket by id and account_id.
pub async fn create_bucket(
    account_id: &str,
    name: &str,
    r#type: &BucketType,
    storage_class: &StorageClass,
    database: &Database,
) -> Result<CreatedResource, sqlx::Error> {
    sqlx::query_as!(
        models::CreatedResource,
        r#"INSERT INTO buckets (account_id, name, type, storage_class) VALUES ($1, $2, $3, $4) RETURNING id;"#,
        account_id,
        name,
        r#type,
        storage_class,
    )
    .fetch_one(database)
    .await
}

pub async fn select_storage_host(database: &Database) -> Result<models::StorageHost, sqlx::Error> {
    sqlx::query_as!(
        models::StorageHost,
        r#"SELECT id, name, url, used_storage, available_storage, fingerprint, pem FROM storage_hosts ORDER BY RANDOM() LIMIT 1;"#,
    )
    .fetch_one(database)
    .await
}

pub async fn record_storage_grant(
    storage_host_id: &str,
    account_id: &str,
    metadata_id: &str,
    authorized_usage: u64,
    database: &Database,
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
    .fetch_one(database)
    .await?;

    sqlx::query!(r#"
            INSERT INTO
                storage_hosts_metadatas_storage_grants (storage_host_id, metadata_id, storage_grant_id)
                VALUES ($1, $2, $3);"#,
            storage_host_id,
            metadata_id,
            storage_grant_id,
        )
        .execute(database)
        .await?;

    Ok(storage_grant_id)
}

/// Pull the bucket from the database by id and account_id and return it. Implements an authorized
/// read of a bucket by id and account_id.
pub async fn read_bucket(
    account_id: &str,
    bucket_id: &str,
    database: &Database,
) -> Result<models::Bucket, sqlx::Error> {
    sqlx::query_as!(
        models::Bucket,
        r#"SELECT id, account_id, name, type, storage_class FROM buckets WHERE id = $1 AND account_id = $2"#,
        bucket_id,
        account_id,
    )
    .fetch_one(database)
    .await
}

/// Pull all buckets from the database by account_id and return them. Implements an authorized read
/// of all buckets by account_id.
pub async fn read_all_buckets(
    account_id: &str,
    database: &Database,
) -> Result<Vec<models::Bucket>, sqlx::Error> {
    sqlx::query_as!(
        models::Bucket,
        r#"SELECT id, account_id, name, type, storage_class FROM buckets WHERE account_id = $1"#,
        account_id,
    )
    .fetch_all(database)
    .await
}

/// Delete a bucket by id and account_id and return it.
/// Implements an authorized delete of a bucket by id and account_id.
pub async fn delete_bucket(
    account_id: &str,
    bucket_id: &str,
    database: &Database,
) -> Result<(), sqlx::Error> {
    // delete does not tell us whether any rows existed, to return a 404 we need to see if its
    // present or not. We'll cheat and use our read bucket method for this 404 check.
    read_bucket(account_id, bucket_id, database).await?;

    sqlx::query!(
        r#"DELETE FROM buckets WHERE id = $1 AND account_id = $2"#,
        bucket_id,
        account_id,
    )
    .execute(database)
    .await?;

    Ok(())
}

/// Create a bucket key by its id and PEM
pub async fn create_bucket_key(
    bucket_id: &str,
    approved: bool,
    pem: &str,
    database: &Database,
) -> Result<CreatedResource, sqlx::Error> {
    let fingerprint = fingerprint_public_pem(pem);
    tracing::info!("creating new bucketkey w fingerprint {fingerprint}");
    sqlx::query_as!(
        models::CreatedResource,
        r#"INSERT INTO bucket_keys (bucket_id, approved, pem, fingerprint) VALUES ($1, $2, $3, $4) RETURNING id;"#,
        bucket_id,
        approved,
        pem,
        fingerprint
    )
    .fetch_one(database)
    .await
}

/// Read a bucket key by its id and authorize that it belongs to a given bucket_id.
pub async fn read_bucket_key(
    bucket_id: &str,
    bucket_key_id: &str,
    database: &Database,
) -> Result<models::BucketKey, sqlx::Error> {
    sqlx::query_as!(
        models::BucketKey,
        r#"SELECT id, bucket_id, approved, pem, fingerprint FROM bucket_keys WHERE id = $1 AND bucket_id = $2;"#,
        bucket_key_id,
        bucket_id,
    )
    .fetch_one(database)
    .await
}

/// Read all bucket keys by a given bucket_id.
pub async fn read_all_bucket_keys(
    bucket_id: &str,
    database: &Database,
) -> Result<Vec<models::BucketKey>, sqlx::Error> {
    sqlx::query_as!(
        models::BucketKey,
        r#"SELECT id, bucket_id, approved, pem, fingerprint FROM bucket_keys WHERE bucket_id = $1;"#,
        bucket_id,
    )
    .fetch_all(database)
    .await
}

/// Delete a bucket key by its id and authorize that it belongs to a given bucket_id.
pub async fn delete_bucket_key(
    bucket_id: &str,
    bucket_key_id: &str,
    database: &Database,
) -> Result<models::BucketKey, sqlx::Error> {
    sqlx::query_as!(
        models::BucketKey,
        r#"DELETE FROM bucket_keys WHERE id = $1 AND bucket_id = $2 RETURNING id, bucket_id, approved, pem, fingerprint;"#,
        bucket_key_id,
        bucket_id,
    )
    .fetch_one(database)
    .await
}

/// Approve a bucket key for use by its id and authorize that it belongs to a given bucket_id.
pub async fn approve_bucket_key(
    bucket_id: &str,
    fingerprint: &str,
    database: &Database,
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
    .fetch_one(database)
    .await
}

/// Read metadata from the database, checking if references a given bucket_id.
pub async fn read_metadata(
    bucket_id: &str,
    metadata_id: &str,
    database: &Database,
) -> Result<models::MetadataWithSnapshot, sqlx::Error> {
    let maybe_metadata = sqlx::query_as!(
        models::Metadata,
        r#"SELECT id, bucket_id, root_cid, metadata_cid, expected_data_size, data_size as "data_size!", state, metadata_size as "metadata_size!", metadata_hash as "metadata_hash!", created_at, updated_at
        FROM metadata WHERE id = $1 AND bucket_id = $2;"#,
        metadata_id,
        bucket_id,
    )
    .fetch_one(database)
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
    .fetch_optional(database)
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
pub async fn authorize_metadata(
    bucket_id: &str,
    metadata_id: &str,
    database: &Database,
) -> Result<(), sqlx::Error> {
    sqlx::query_as!(
        models::CreatedResource,
        r#"SELECT id FROM metadata WHERE id = $1 AND bucket_id = $2;"#,
        metadata_id,
        bucket_id,
    )
    .fetch_one(database)
    .await
    .map(|_| ())
}

/// Read all metadata from the database by a given bucket_id.
pub async fn read_all_metadata(
    bucket_id: &str,
    database: &Database,
) -> Result<Vec<models::Metadata>, sqlx::Error> {
    sqlx::query_as!(
        models::Metadata,
        r#"SELECT id, bucket_id, root_cid, metadata_cid, expected_data_size, data_size as "data_size!", state, metadata_size as "metadata_size!", metadata_hash as "metadata_hash!", created_at, updated_at
        FROM metadata WHERE bucket_id = $1;"#,
        bucket_id,
    )
    .fetch_all(database)
    .await
}

/// Read the current metadata from the database by a given bucket_id.
pub async fn read_current_metadata(
    bucket_id: &str,
    database: &Database,
) -> Result<models::MetadataWithSnapshot, sqlx::Error> {
    let maybe_metadata = sqlx::query_as!(
        models::Metadata,
        r#"SELECT id, bucket_id, root_cid, metadata_cid, expected_data_size, data_size as "data_size!", state, metadata_size as "metadata_size!", metadata_hash as "metadata_hash!", created_at, updated_at
        FROM metadata WHERE bucket_id = $1 AND state = 'current';"#,
        bucket_id,
    )
    .fetch_one(database)
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
    .fetch_optional(database)
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

/// Create a snapshot and return the created resource.
pub async fn create_snapshot(
    metadata_id: &str,
    database: &Database,
) -> Result<models::CreateSnapshot, sqlx::Error> {
    sqlx::query_as!(
        models::CreateSnapshot,
        r#"INSERT INTO snapshots (metadata_id)
        VALUES ($1)
        RETURNING id, created_at;"#,
        metadata_id
    )
    .fetch_one(database)
    .await
}

/// Read a snapshot by its id and authorize that its associated metadata belongs to a given bucket_id.
pub async fn read_snapshot(
    bucket_id: &str,
    snapshot_id: &str,
    database: &Database,
) -> Result<models::Snapshot, sqlx::Error> {
    sqlx::query_as!(
        models::Snapshot,
        r#"SELECT 
            s.id,
            s.metadata_id as "metadata_id!",
            m.data_size as "size!",
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
    .fetch_one(database)
    .await
}

/// Read a snapshot by bucket_id and snapshot_id.
pub async fn read_all_snapshots(
    bucket_id: &str,
    database: &Database,
) -> Result<Vec<models::Snapshot>, sqlx::Error> {
    sqlx::query_as!(
        models::Snapshot,
        r#"SELECT 
            s.id,
            s.metadata_id as "metadata_id!",
            m.data_size as "size!",
            s.created_at as "created_at!"
        FROM 
            snapshots s
        INNER JOIN 
            metadata m ON m.id = s.metadata_id
        WHERE 
            m.bucket_id = $1;"#,
        bucket_id
    )
    .fetch_all(database)
    .await
}

/// Read the data usage of a given bucket id.
pub async fn read_bucket_data_usage(
    bucket_id: &str,
    database: &Database,
) -> Result<u64, sqlx::Error> {
    let maybe_data_usage = sqlx::query_as!(
        GetUsage,
        r#"SELECT
                    COALESCE(SUM(m.data_size), 0) as "size!"
                FROM
                    metadata m
                WHERE
                    m.bucket_id = $1 AND m.state IN ('pending', 'current');"#,
        bucket_id,
    )
    .fetch_one(database)
    .await;
    match maybe_data_usage {
        Ok(usage) => Ok(usage.size as u64),
        Err(err) => match err {
            sqlx::Error::RowNotFound => Err(sqlx::Error::RowNotFound),
            _ => Err(err),
        },
    }
}
