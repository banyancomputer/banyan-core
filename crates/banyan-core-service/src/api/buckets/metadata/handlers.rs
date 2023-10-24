use axum::body::StreamBody;
use axum::extract::{BodyStream, Path};
use axum::headers::ContentType;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use axum::TypedHeader;
use http::{HeaderMap, HeaderValue};
use object_store::ObjectStore;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

use crate::api::buckets::metadata::{requests, responses};
use crate::db::models;
use crate::error::CoreError;
use crate::extractors::{ApiToken, ApiTokenKid, DataStore, DbConn, SigningKey};
use crate::utils::db::{self, approve_bucket_key};
use crate::utils::metadata_upload::{handle_metadata_upload, round_to_nearest_100_mib};
use crate::utils::storage_ticket::generate_storage_ticket;

/// Usage limit for all accounts (5 TiB)
const ACCOUNT_STORAGE_QUOTA: u64 = 5 * 1_024 * 1_024 * 1_024 * 1_024;

/// Upload data size limit for CAR file uploads
const REQUEST_DATA_SIZE_LIMIT: u64 = 100 * 1_024;

/// Upload size limit for CAR files
const CAR_DATA_SIZE_LIMIT: u64 = 128 * 1_024 * 1_024;

/// Handle a request to push new metadata to a bucket
#[allow(clippy::too_many_arguments)]
pub async fn push(
    api_token: ApiToken,
    api_token_kid: ApiTokenKid,
    mut db_conn: DbConn,
    store: DataStore,
    signing_key: SigningKey,
    Path(bucket_id): Path<Uuid>,
    TypedHeader(content_type): TypedHeader<ContentType>,
    body: BodyStream,
) -> impl IntoResponse {
    let account_id = api_token.subject;
    let bucket_id = bucket_id.to_string();
    let api_token_kid = api_token_kid.kid();

    /* 1. Authorize access to the bucket and validate the request */
    if let Err(err) = db::authorize_bucket(&account_id, &bucket_id, &mut db_conn).await {
        return CoreError::sqlx_error(err, "read", "bucket").into_response();
    }

    // TODO: Check if the uploaded version exists. If-Match matches existing version abort with 409

    // Read the body from the request, checking for size limits
    let mime_ct = mime::Mime::from(content_type);
    let boundary = multer::parse_boundary(mime_ct).unwrap();
    let constraints = multer::Constraints::new()
        .allowed_fields(vec!["request-data", "car-upload"])
        .size_limit(
            multer::SizeLimit::new()
                .for_field("request-data", REQUEST_DATA_SIZE_LIMIT)
                .for_field("car-upload", CAR_DATA_SIZE_LIMIT),
        );
    let mut multipart = multer::Multipart::with_constraints(body, boundary, constraints);

    // Process the request data
    let request_data_field = multipart.next_field().await.unwrap().unwrap();
    // TODO: validate name is request-data (request_data_field.name())
    // TODO: validate type is application/json (request_data_field.content_type())
    let request_data_bytes = request_data_field.bytes().await.unwrap();
    let request_data: requests::PushMetadataRequest =
        match serde_json::from_slice(&request_data_bytes) {
            Ok(rdb) => rdb,
            Err(err) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({"msg": format!("{err}")})),
                )
                    .into_response();
            }
        };

    /* 2. Now that the request is validated and the data extracted, approve any outstanding keys */
    for fingerprint in request_data.valid_keys {
        // Return if we fail to approve any of them
        if let Err(err) = approve_bucket_key(&bucket_id, &fingerprint, &mut db_conn).await {
            return CoreError::sqlx_error(err, "approve", "bucket key").into_response();
        }
    }

    /* 2. Create a tentative row for the new metadata. We need to do this in order to get a created resource */

    let expected_data_size = request_data.expected_data_size as i64;
    let maybe_metadata_resource = sqlx::query_as!(
        models::CreatedResource,
        r#"INSERT INTO metadata (bucket_id, root_cid, metadata_cid, expected_data_size, state)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id;"#,
        bucket_id,
        request_data.root_cid,
        request_data.metadata_cid,
        expected_data_size,
        models::MetadataState::Uploading
    )
    .fetch_one(&mut *db_conn.0)
    .await;

    let metadata_resource = match maybe_metadata_resource {
        Ok(metadata) => metadata,
        Err(err) => {
            return CoreError::sqlx_error(err, "create", "metadata").into_response();
        }
    };

    /* 3. Process the upload */

    let car_stream = multipart.next_field().await.unwrap().unwrap();
    // TODO: validate name is car-upload (request_data_field.name())
    // TODO: validate type is "application/vnd.ipld.car; version=2" (request_data_field.content_type())
    let file_name = format!(
        "{bucket_id}/{metadata_id}.car",
        bucket_id = bucket_id,
        metadata_id = metadata_resource.id
    );
    let file_path = object_store::path::Path::from(file_name.as_str());
    // Open the file for writing
    let (upload_id, mut writer) = match store.put_multipart(&file_path).await {
        // If we created the writer, go ahead and do the upload
        Ok(mp) => mp,
        // Otherwise, try marking the update as failed
        Err(_) => {
            tracing::error!(
                "could not open writer to metadata file <id: {}>",
                metadata_resource.id
            );
            // Try and mark the upload as failed
            let maybe_failed_metadata_upload = sqlx::query!(
                r#"UPDATE metadata SET state = $1 WHERE id = $2;"#,
                models::MetadataState::UploadFailed,
                metadata_resource.id
            )
            .execute(&mut *db_conn.0)
            .await;
            // Return the correct response based on the result of the update
            return match maybe_failed_metadata_upload {
                Ok(_) => CoreError::default_error("failed to upload metadata"),
                Err(err) => CoreError::sqlx_error(err, "mark failed", "metadata upload"),
            }
            .into_response();
        }
    };
    // Try and upload the file
    let (metadata_hash, metadata_size) = match handle_metadata_upload(car_stream, &mut writer).await
    {
        Ok(fh) => {
            writer
                .shutdown()
                .await
                .expect("upload finalization to succeed");
            fh
        }
        Err(_) => {
            store
                .abort_multipart(&file_path, &upload_id)
                .await
                .expect("aborting to success");
            return CoreError::default_error("unable to process upload").into_response();
        }
    };

    /* 4. Now that we know the size of metadata, Check if the upload exceeds the user's storage quota. If so, abort with 413 */

    // Read how metadata and data the use has in the current and pending states across all buckets
    let current_usage = match db::read_total_usage(&account_id, &mut db_conn).await {
        Ok(usage) => usage,
        Err(err) => {
            return CoreError::default_error(&format!(
                "unable to read account storage usage: {err}"
            ))
            .into_response();
        }
    };

    tracing::info!(metadata_id = ?metadata_resource.id, current_usage = ?current_usage, data_size = ?expected_data_size, meta_size = ?metadata_size, "created new metadata entry");

    // Based on how much stuff there planning on pushing, reject the upload if it would exceed the quota
    // Expected usage is their current usage plus the size of the metadata they're uploading plus the size of the data they want to upload to a host
    let expected_usage = current_usage + metadata_size + expected_data_size as u64;
    if expected_usage > ACCOUNT_STORAGE_QUOTA {
        // Mark the upload as failed
        let maybe_failed_metadata_upload = sqlx::query!(
            r#"UPDATE metadata SET state = $1 WHERE id = $2;"#,
            models::MetadataState::UploadFailed,
            metadata_resource.id
        )
        .execute(&mut *db_conn.0)
        .await;
        match maybe_failed_metadata_upload {
            Ok(_) => {}
            Err(err) => {
                return CoreError::default_error(&format!(
                    "unable to mark metadata upload as failed: {}",
                    err
                ))
                .into_response();
            }
        };
        // Return the correct response based on the result of the update
        return (
            StatusCode::PAYLOAD_TOO_LARGE,
            format!(
                "account storage quota exceeded: {current_usage} + {request_size} > {ACCOUNT_STORAGE_QUOTA}",
                current_usage = current_usage,
                request_size = expected_data_size + metadata_size as i64,
                ACCOUNT_STORAGE_QUOTA = ACCOUNT_STORAGE_QUOTA
            ),
        )
            .into_response();
    }

    /* 5. Ah yes! They can indeed store this data. Mark the upload as complete and put it in the appropriate state */

    // Check that the user is actually asking for more data in this request.
    // If not, update the metadata state to current and return a proper response
    // If so, update the metadata state to pending and continue
    if expected_data_size == 0 {
        let current_metadata_state = models::MetadataState::Current.to_string();
        let metadata_size = metadata_size as i64;
        let expected_data_size = expected_data_size as i64;
        let maybe_current_metadata = sqlx::query_as!(
            models::CreatedResource,
            r#"UPDATE metadata SET state = $1, metadata_size = $2, data_size = $3, metadata_hash = $4 WHERE id = $5 RETURNING id;"#,
            current_metadata_state,
            metadata_size,
            expected_data_size,
            metadata_hash,
            metadata_resource.id
        ).fetch_one(&mut *db_conn.0).await;
        let current_metadata = match maybe_current_metadata {
            Ok(cr) => cr,
            Err(err) => {
                return CoreError::default_error(&format!(
                    "unable to update bucket metadata after push: {err}"
                ))
                .into_response();
            }
        };
        // Set all current metadata to outdated, except for the one we just uploaded
        let outdated_metadata_state = models::MetadataState::Outdated.to_string();
        let maybe_outdated_metadata = sqlx::query!(
            r#"UPDATE metadata SET state = $1 WHERE bucket_id = $2 AND id != $3 AND state = $4;"#,
            outdated_metadata_state,
            bucket_id,
            metadata_resource.id,
            current_metadata_state
        )
        .execute(&mut *db_conn.0)
        .await;
        match maybe_outdated_metadata {
            Ok(_) => {
                return (
                    StatusCode::OK,
                    axum::Json(responses::PushMetadataResponse {
                        id: current_metadata.id.to_string(),
                        state: models::MetadataState::Current,
                        storage_host: None,
                        storage_authorization: None,
                    }),
                )
                    .into_response()
            }
            Err(err) => {
                return CoreError::default_error(&format!(
                    "unable to update bucket metadata after push: {err}"
                ))
                .into_response();
            }
        }
    }
    // OK, they're actually asking for more data. Update the metadata state to pending and continue
    let metadata_state = models::MetadataState::Pending.to_string();
    let metadata_size = metadata_size as i64;
    let maybe_updated_metadata = sqlx::query_as!(
        models::CreatedResource,
        r#"UPDATE metadata SET state = $1, metadata_size = $2, metadata_hash = $3 WHERE id = $4 RETURNING id;"#,
        metadata_state,
        metadata_size,
        metadata_hash,
        metadata_resource.id
    )
    .fetch_one(&mut *db_conn.0)
    .await;
    let updated_metadata = match maybe_updated_metadata {
        Ok(cr) => cr,
        Err(err) => {
            return CoreError::default_error(&format!(
                "unable to update bucket metadata after push: {err}"
            ))
            .into_response();
        }
    };

    /* 6. Determine a storage host we can point them too. Determine what they're expected usage on that host will be after upload */

    // Round up to the nearest 100 MiB
    let data_authorization = round_to_nearest_100_mib(expected_usage);
    tracing::info!(account_id = ?account_id, authorized_amt = ?data_authorization, "authorizing user more storage");

    // Read a storage host from the database. We only have one right now, so this is easy
    let storage_host = match db::select_storage_host(&mut db_conn).await {
        Ok(sh) => sh,
        Err(err) => {
            return CoreError::default_error(&format!("unable to read storage host: {err}"))
                .into_response();
        }
    };
    // TODO: Check if the storage host is full. If so, abort with 503

    /* 7. Generate a JWT for the storage host and return it to the user */
    let storage_grant_id = match db::record_storage_grant(
        &storage_host.id,
        &account_id,
        &metadata_resource.id,
        data_authorization,
        &mut db_conn,
    )
    .await
    {
        Ok(sgi) => sgi,
        Err(err) => {
            return CoreError::default_error(&format!("unable record storage grant: {err}"))
                .into_response();
        }
    };

    let storage_authorization = match generate_storage_ticket(
        &account_id,
        &storage_grant_id,
        api_token_kid,
        &storage_host.name,
        &storage_host.url,
        data_authorization,
        &signing_key,
    ) {
        Ok(ticket) => ticket,
        Err(err) => {
            return CoreError::default_error(&format!("unable to generate storage ticket: {err}"))
                .into_response();
        }
    };

    let response = responses::PushMetadataResponse {
        id: updated_metadata.id.to_string(),
        state: models::MetadataState::Pending,
        storage_host: Some(storage_host.url),
        storage_authorization: Some(storage_authorization),
    };

    (StatusCode::OK, axum::Json(response)).into_response()
}

/// Handle a request to pull metadata from a bucket
pub async fn pull(
    api_token: ApiToken,
    mut db_conn: DbConn,
    store: DataStore,
    Path((bucket_id, metadata_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    let account_id = api_token.subject;
    let bucket_id = bucket_id.to_string();
    let metadata_id = metadata_id.to_string();
    match db::authorize_bucket(&account_id, &bucket_id, &mut db_conn).await {
        Ok(_) => {}
        Err(err) => match err {
            sqlx::Error::RowNotFound => {
                return (StatusCode::NOT_FOUND, format!("bucket not found: {err}")).into_response();
            }
            _ => {
                return CoreError::default_error(&format!("unable to read bucket: {err}"))
                    .into_response();
            }
        },
    };
    // Make sure the metadata exists
    match db::authorize_metadata(&bucket_id, &metadata_id, &mut db_conn).await {
        Ok(_) => {}
        Err(err) => match err {
            sqlx::Error::RowNotFound => {
                return (StatusCode::NOT_FOUND, format!("metadata not found: {err}"))
                    .into_response();
            }
            _ => {
                return CoreError::default_error(&format!("unable to read metadata: {err}"))
                    .into_response();
            }
        },
    };

    // Try opening the file for reading
    let file_name = format!(
        "{bucket_id}/{metadata_id}.car",
        bucket_id = bucket_id,
        metadata_id = metadata_id
    );
    let file_path = object_store::path::Path::from(file_name.as_str());
    let reader = match store.get(&file_path).await {
        Ok(r) => r,
        Err(err) => {
            return CoreError::default_error(&format!("unable to read metadata file: {err}"))
                .into_response();
        }
    };
    let stream = reader.into_stream();

    // Create the headers for the response
    let mut headers = HeaderMap::new();

    headers.insert(
        http::header::CONTENT_TYPE,
        HeaderValue::from_static("application/vnd.ipld.car; version=2"),
    );
    headers.insert(
        http::header::CONTENT_DISPOSITION,
        HeaderValue::from_str(format!("attachment; filename=\"{file_name}\"").as_str()).unwrap(),
    );

    let body = StreamBody::new(stream);

    (StatusCode::OK, headers, body).into_response()
}

pub async fn read(
    api_token: ApiToken,
    mut db_conn: DbConn,
    Path((bucket_id, metadata_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    let account_id = api_token.subject;
    let bucket_id = bucket_id.to_string();
    let metadata_id = metadata_id.to_string();
    match db::authorize_bucket(&account_id, &bucket_id, &mut db_conn).await {
        Ok(_) => {}
        Err(err) => match err {
            sqlx::Error::RowNotFound => {
                return (StatusCode::NOT_FOUND, format!("bucket not found: {err}")).into_response();
            }
            _ => {
                return CoreError::default_error(&format!("unable to read bucket: {err}"))
                    .into_response();
            }
        },
    };
    // Read the metadata
    let response = match db::read_metadata(&bucket_id, &metadata_id, &mut db_conn).await {
        Ok(bm) => responses::ReadMetadataResponse {
            id: bm.metadata.id.to_string(),
            root_cid: bm.metadata.root_cid,
            metadata_cid: bm.metadata.metadata_cid,
            data_size: bm.metadata.data_size,
            state: bm.metadata.state,
            created_at: bm.metadata.created_at.unix_timestamp(),
            updated_at: bm.metadata.updated_at.unix_timestamp(),
            snapshot_id: bm.snapshot_id,
        },
        Err(err) => match err {
            sqlx::Error::RowNotFound => {
                return (StatusCode::NOT_FOUND, format!("metadata not found: {err}"))
                    .into_response();
            }
            _ => {
                return CoreError::default_error(&format!("unable to read metadata: {err}"))
                    .into_response();
            }
        },
    };
    (StatusCode::OK, axum::Json(response)).into_response()
}

/// Read all uploaded metadata for a bucket
pub async fn read_all(
    api_token: ApiToken,
    mut db_conn: DbConn,
    Path(bucket_id): Path<Uuid>,
) -> impl IntoResponse {
    let account_id = api_token.subject;
    let bucket_id = bucket_id.to_string();
    match db::authorize_bucket(&account_id, &bucket_id, &mut db_conn).await {
        Ok(_) => {}
        Err(err) => match err {
            sqlx::Error::RowNotFound => {
                return (StatusCode::NOT_FOUND, format!("bucket not found: {err}")).into_response();
            }
            _ => {
                return CoreError::default_error(&format!("unable to read bucket: {err}"))
                    .into_response();
            }
        },
    };
    let response = match db::read_all_metadata(&bucket_id, &mut db_conn).await {
        Ok(bm) => responses::ReadAllMetadataResponse(
            bm.into_iter()
                .map(|bm| responses::ReadMetadataResponse {
                    id: bm.id.to_string(),
                    root_cid: bm.root_cid,
                    metadata_cid: bm.metadata_cid,
                    data_size: bm.data_size,
                    state: bm.state,
                    created_at: bm.created_at.unix_timestamp(),
                    updated_at: bm.updated_at.unix_timestamp(),
                    snapshot_id: None,
                })
                .collect(),
        ),
        Err(err) => match err {
            sqlx::Error::RowNotFound => {
                return (StatusCode::NOT_FOUND, format!("metadata not found: {err}"))
                    .into_response();
            }
            _ => {
                return CoreError::default_error(&format!("unable to read metadata: {err}"))
                    .into_response();
            }
        },
    };
    (StatusCode::OK, axum::Json(response)).into_response()
}

/// Read the current metadata for a bucket or return 404 if there is no current metadata
pub async fn read_current(
    api_token: ApiToken,
    mut db_conn: DbConn,
    Path(bucket_id): Path<Uuid>,
) -> impl IntoResponse {
    let account_id = api_token.subject;
    let bucket_id = bucket_id.to_string();
    match db::authorize_bucket(&account_id, &bucket_id, &mut db_conn).await {
        Ok(_) => {}
        Err(err) => match err {
            sqlx::Error::RowNotFound => {
                return (StatusCode::NOT_FOUND, format!("bucket not found: {err}")).into_response();
            }
            _ => {
                return CoreError::default_error(&format!("unable to read bucket: {err}"))
                    .into_response();
            }
        },
    };
    let response = match db::read_current_metadata(&bucket_id, &mut db_conn).await {
        Ok(bm) => responses::ReadMetadataResponse {
            id: bm.metadata.id.to_string(),
            root_cid: bm.metadata.root_cid,
            metadata_cid: bm.metadata.metadata_cid,
            data_size: bm.metadata.data_size,
            state: bm.metadata.state,
            created_at: bm.metadata.created_at.unix_timestamp(),
            updated_at: bm.metadata.updated_at.unix_timestamp(),
            snapshot_id: bm.snapshot_id,
        },
        Err(err) => match err {
            sqlx::Error::RowNotFound => {
                return (
                    StatusCode::NOT_FOUND,
                    format!("current metadata not found: {err}"),
                )
                    .into_response();
            }
            _ => {
                return CoreError::default_error(&format!("unable to read bucket metadata: {err}"))
                    .into_response();
            }
        },
    };

    (StatusCode::OK, axum::Json(response)).into_response()
}

pub async fn delete(
    _api_token: ApiToken,
    Path(_bucket_id): Path<Uuid>,
    Path(_metadata_id): Path<Uuid>,
) -> impl IntoResponse {
    // TODO: Implement
    (StatusCode::NO_CONTENT, ()).into_response()
}
