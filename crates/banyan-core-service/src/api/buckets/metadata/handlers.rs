use axum::body::StreamBody;
use axum::extract::{BodyStream, Json, Path};
use axum::headers::ContentType;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::TypedHeader;
use http::{HeaderMap, HeaderValue};
use object_store::ObjectStore;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

use crate::api::buckets::metadata::{requests, responses};
use crate::db::models;
use crate::extractors::{ApiToken, DataStore, DbConn, StorageHost};
use crate::utils::metadata_upload::handle_metadata_upload;

/// Upload data size limit for CAR file uploads
const REQUEST_DATA_SIZE_LIMIT: u64 = 100 * 1_024;
/// Upload size limit for CAR files
const CAR_DATA_SIZE_LIMIT: u64 = 128 * 1_024 * 1_024;

/// Handle a request to push new metadata to a bucket
pub async fn push(
    api_token: ApiToken,
    mut db_conn: DbConn,
    store: DataStore,
    storage_host: StorageHost,
    Path(bucket_id): Path<Uuid>,
    TypedHeader(content_type): TypedHeader<ContentType>,
    body: BodyStream,
) -> impl IntoResponse {
    let account_id = api_token.subject;
    let bucket_id = bucket_id.to_string();
    // Make sure the calling user owns the bucket
    let maybe_bucket = sqlx::query_as!(
        models::CreatedResource,
        r#"SELECT id as "id!" FROM buckets WHERE id = $1 AND account_id = $2;"#,
        bucket_id,
        account_id
    )
    .fetch_one(&mut *db_conn.0)
    .await;

    match maybe_bucket {
        Ok(b) => b,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("unable to read bucket: {err}"),
            )
                .into_response();
        }
    };

    // TODO: Check if the uploaded version exists. If-Match matches existing version abort with 409
    // TODO: Check if the upload exceeds the user's storage quota. If so, abort with 413

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
        serde_json::from_slice(&request_data_bytes).unwrap();
    // TODO: Validata that the account is allowed to store `request_data.data_size` bytes

    let i_size = request_data.data_size as i64;
    let maybe_metadata_resource = sqlx::query_as!(
        models::CreatedResource,
        r#"INSERT INTO metadata (bucket_id, root_cid, metadata_cid, data_size, state)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id as "id!";"#,
        bucket_id,
        request_data.root_cid,
        request_data.metadata_cid,
        i_size,
        models::MetadataState::Uploading
    )
    .fetch_one(&mut *db_conn.0)
    .await;

    let metadata_resource = match maybe_metadata_resource {
        Ok(bm) => bm,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("unable to create bucket metadata: {err}"),
            )
                .into_response();
        }
    };

    // Process the upload
    let car_stream = multipart.next_field().await.unwrap().unwrap();
    // TODO: validate name is car-upload (request_data_field.name())
    // TODO: validate type is "application/vnd.ipld.car; version=2" (request_data_field.content_type())

    let file_name = format!(
        "{bucket_id}/{metadata_id}.car",
        bucket_id = bucket_id,
        metadata_id = metadata_resource.id
    );
    let file_path = object_store::path::Path::from(file_name.as_str());

    let (upload_id, mut writer) = match store.put_multipart(&file_path).await {
        // If we created the writer, go ahead and do the upload
        Ok(mp) => mp,
        // Otherwise, try marking the update as failed
        Err(_) => {
            // Try and mark the upload as failed
            let maybe_failed_metadata_upload = sqlx::query!(
                r#"UPDATE metadata SET state = $1 WHERE id = $2;"#,
                models::MetadataState::UploadFailed,
                metadata_resource.id
            )
            .execute(&mut *db_conn.0)
            .await;

            // Return the correct response based on the result of the update
            match maybe_failed_metadata_upload {
                Ok(_) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(responses::PushMetadataResponse {
                            id: metadata_resource.id.to_string(),
                            state: models::MetadataState::UploadFailed,
                            storage_host: "N/A".to_string(),
                            storage_authorization: "N/A".to_string(),
                        }),
                    )
                        .into_response();
                }
                Err(err) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("unable to update bucket metadata: {err}"),
                    )
                        .into_response();
                }
            };
        }
    };

    // Try and upload the file
    let (hash, size) = match handle_metadata_upload(car_stream, &mut writer).await {
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
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "unable to process upload",
            )
                .into_response();
        }
    };

    // Mark the upload as complete and fill in the size and hash
    let i_size = size as i64;
    // TODO: Eventually this should use pending
    // let bucket_state = models::MetadataState::Pending;
    let bucket_state = models::MetadataState::Current;
    let bucket_state_string = bucket_state.to_string();
    let maybe_updated_metadata = sqlx::query_as!(
        models::CreatedResource,
        r#"UPDATE metadata SET state = $1, size = $2, hash = $3 WHERE id = $4 RETURNING id as "id!";"#,
        bucket_state_string,
        i_size,
        hash,
        metadata_resource.id
    )
    .fetch_one(&mut *db_conn.0)
    .await;

    let updated_metadata = match maybe_updated_metadata {
        Ok(bm) => bm,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("unable to update bucket metadata: {err}"),
            )
                .into_response();
        }
    };

    let response = responses::PushMetadataResponse {
        id: updated_metadata.id.to_string(),
        state: models::MetadataState::Pending,
        storage_host: storage_host.as_string(),
        // TODO: this should be a JWT
        storage_authorization: "TODO: JWT here".to_string(),
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
    let maybe_bucket = sqlx::query_as!(
        models::CreatedResource,
        r#"SELECT id as "id!" FROM buckets WHERE id = $1 AND account_id = $2;"#,
        bucket_id,
        account_id
    )
    .fetch_one(&mut *db_conn.0)
    .await;
    match maybe_bucket {
        Ok(b) => b,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("unable to read bucket: {err}"),
            )
                .into_response();
        }
    };
    // Make sure the metadata exists
    let maybe_metadata = sqlx::query_as!(
        models::CreatedResource,
        r#"SELECT id as "id!" FROM metadata WHERE id = $1 AND bucket_id = $2;"#,
        metadata_id,
        bucket_id
    )
    .fetch_one(&mut *db_conn.0)
    .await;
    match maybe_metadata {
        Ok(bm) => bm,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("unable to read bucket metadata: {err}"),
            )
                .into_response();
        }
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
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("unable to read bucket: {err}"),
            )
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
    let maybe_bucket = sqlx::query_as!(
        models::CreatedResource,
        r#"SELECT id as "id!" FROM buckets WHERE id = $1 AND account_id = $2;"#,
        bucket_id,
        account_id
    )
    .fetch_one(&mut *db_conn.0)
    .await;
    match maybe_bucket {
        Ok(b) => b,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("unable to read bucket: {err}"),
            )
                .into_response();
        }
    };
    // Make sure the metadata exists
    let maybe_metadata = sqlx::query_as!(
        models::Metadata,
        r#"SELECT id as "id!", bucket_id, root_cid, metadata_cid, data_size, state, size as "size!", hash as "hash!", created_at, updated_at
        FROM metadata WHERE id = $1 AND bucket_id = $2;"#,
        metadata_id,
        bucket_id
    )
    .fetch_one(&mut *db_conn.0)
    .await;

    let metadata = match maybe_metadata {
        Ok(bm) => responses::ReadMetadataResponse {
            id: bm.id.to_string(),
            root_cid: bm.root_cid,
            metadata_cid: bm.metadata_cid,
            data_size: bm.data_size,
            state: bm.state,
            created_at: bm.created_at.timestamp(),
            updated_at: bm.updated_at.timestamp(),
        },
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("unable to read bucket metadata: {err}"),
            )
                .into_response();
        }
    };

    (StatusCode::OK, axum::Json(metadata)).into_response()
}

/// Read all uploaded metadata for a bucket
pub async fn read_all(
    api_token: ApiToken,
    mut db_conn: DbConn,
    Path(bucket_id): Path<Uuid>,
) -> impl IntoResponse {
    let account_id = api_token.subject;
    let bucket_id = bucket_id.to_string();
    let maybe_bucket = sqlx::query_as!(
        models::CreatedResource,
        r#"SELECT id as "id!" FROM buckets WHERE id = $1 AND account_id = $2;"#,
        bucket_id,
        account_id
    )
    .fetch_one(&mut *db_conn.0)
    .await;
    match maybe_bucket {
        Ok(b) => b,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("unable to read bucket: {err}"),
            )
                .into_response();
        }
    };
    // Make sure the metadata exists
    let maybe_metadata = sqlx::query_as!(
        models::Metadata,
        r#"SELECT id as "id!", bucket_id, root_cid, metadata_cid, data_size, state, size as "size!", hash as "hash!", created_at, updated_at
        FROM metadata WHERE bucket_id = $1;"#,
        bucket_id
    )
    .fetch_all(&mut *db_conn.0)
    .await;

    let metadata = match maybe_metadata {
        Ok(bm) => responses::ReadAllMetadataResponse(
            bm.into_iter()
                .map(|bm| responses::ReadMetadataResponse {
                    id: bm.id.to_string(),
                    root_cid: bm.root_cid,
                    metadata_cid: bm.metadata_cid,
                    data_size: bm.data_size,
                    state: bm.state,
                    created_at: bm.created_at.timestamp(),
                    updated_at: bm.updated_at.timestamp(),
                })
                .collect(),
        ),
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("unable to read bucket metadata: {err}"),
            )
                .into_response();
        }
    };

    (StatusCode::OK, axum::Json(metadata)).into_response()
}

pub async fn delete(
    _api_token: ApiToken,
    Path(_bucket_id): Path<Uuid>,
    Path(_metadata_id): Path<Uuid>,
) -> impl IntoResponse {
    // TODO: Implement
    (StatusCode::NO_CONTENT, ()).into_response()
}
