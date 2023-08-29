use std::path::PathBuf;

use axum::extract::{BodyStream, Json};
use axum::headers::{ContentLength, ContentType};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::TypedHeader;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::database::{BareId, DbError, Executor};
use crate::extractors::{AuthenticatedClient, Database, UploadStore};

/// Limit on the size of the JSON request that accompanies an upload.
const UPLOAD_REQUEST_SIZE_LIMIT: u64 = 100 * 1_024;

#[derive(Deserialize, Serialize)]
pub struct UploadRequest {
    metadata_id: Uuid,
}

pub async fn handler(
    db: Database,
    client: AuthenticatedClient,
    _store: UploadStore,
    TypedHeader(content_len): TypedHeader<ContentLength>,
    TypedHeader(content_type): TypedHeader<ContentType>,
    body: BodyStream,
) -> Result<Response, UploadError> {
    let reported_body_length = content_len.0;

    if reported_body_length > client.remaining_storage() {
        return Err(UploadError::InsufficientAuthorizedStorage(
            reported_body_length,
            client.remaining_storage(),
        ));
    }

    let mime_ct = mime::Mime::from(content_type);
    let boundary = multer::parse_boundary(mime_ct).unwrap();
    let constraints = multer::Constraints::new()
        .allowed_fields(vec!["request-data", "car-upload"])
        .size_limit(
            multer::SizeLimit::new()
                .for_field("request-data", UPLOAD_REQUEST_SIZE_LIMIT)
                .for_field("car-upload", client.remaining_storage() as u64),
        );

    let mut multipart = multer::Multipart::with_constraints(body, boundary, constraints);

    let request_data = multipart
        .next_field()
        .await
        .map_err(UploadError::RequestFieldUnavailable)?
        .ok_or(UploadError::RequestFieldMissing)?;

    // TODO: validate name is request-data (request_data_field.name())
    // TODO: validate type is application/json (request_data_field.content_type())

    let request: UploadRequest = request_data
        .json()
        .await
        .map_err(UploadError::InvalidRequestData)?;

    let _upload_id = record_upload_beginning(&db, client.id(), request.metadata_id, reported_body_length).await?;

    // todo: should make sure I have a clean up task that watches for failed uploads and handles
    // them appropriately

    // record that an upload is in progress
    // collect upload in a temporary directory
    // during upload, if it goes over content length warn and start watching remaining authorized
    // storage
    // if upload errors clean up files and record the failure in the database with the uploaded amount
    // if upload succeeds queue task to report back to platform

    Ok((StatusCode::NO_CONTENT, ()).into_response())
}

async fn record_upload_beginning(db: &Database, client_id: Uuid, metadata_id: Uuid, reported_size: u64) -> Result<(Uuid, PathBuf), UploadError> {
    let mut tmp_upload_path = PathBuf::new();

    tmp_upload_path.push("uploading");
    tmp_upload_path.push(client_id.to_string());
    tmp_upload_path.push(format!("{metadata_id}.car"));

    let upload_id: Uuid = match db.ex() {
        #[cfg(feature = "postgres")]
        Executor::Postgres(ref mut conn) => {
            use crate::database::postgres;

            let bare_upload_id: BareId = sqlx::query_as(r#"
                    INSERT INTO
                        uploads (client_id, metadata_id, reported_size, file_path, state)
                        VALUES ($1, $2, $3, $4, 'started')
                        RETURNING id;
                "#)
                .bind(client_id.to_string())
                .bind(metadata_id.to_string())
                .bind(reported_size as i64)
                .bind(tmp_upload_path.display().to_string())
                .fetch_one(conn)
                .await
                .map_err(postgres::map_sqlx_error)
                .map_err(UploadError::Database)?;

            Uuid::parse_str(&bare_upload_id.id).unwrap()
        }

        #[cfg(feature = "sqlite")]
        Executor::Sqlite(ref mut conn) => {
            use crate::database::sqlite;

            let bare_upload_id: BareId = sqlx::query_as(r#"
                    INSERT INTO
                        uploads (client_id, metadata_id, reported_size, file_path, state)
                        VALUES ($1, $2, $3, $4, 'started')
                        RETURNING id;
                "#)
                .bind(client_id.to_string())
                .bind(metadata_id.to_string())
                .bind(reported_size as i64)
                .bind(tmp_upload_path.display().to_string())
                .fetch_one(conn)
                .await
                .map_err(sqlite::map_sqlx_error)
                .map_err(UploadError::Database)?;

            Uuid::parse_str(&bare_upload_id.id).unwrap()
        }
    };

    Ok((upload_id, tmp_upload_path))
}

#[derive(Debug, thiserror::Error)]
pub enum UploadError {
    #[error("a database error occurred during an upload")]
    Database(DbError),

    #[error("account is not authorized to store {0} bytes, {1} bytes are still authorized")]
    InsufficientAuthorizedStorage(u64, u64),

    #[error("request's data payload was malformed")]
    InvalidRequestData(multer::Error),

    #[error("failed to acquire request field from body")]
    RequestFieldUnavailable(multer::Error),

    #[error("we expected a request field but received nothing")]
    RequestFieldMissing,
}

impl IntoResponse for UploadError {
    fn into_response(self) -> Response {
        use UploadError::*;

        match &self {
            Database(_) => {
                tracing::error!("{self}");
                let err_msg = serde_json::json!({ "msg": "a backend service issue occurred" });
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
            }
            InvalidRequestData(_) | RequestFieldUnavailable(_) | RequestFieldMissing => {
                let err_msg = serde_json::json!({ "msg": format!("{self}") });
                (StatusCode::BAD_REQUEST, Json(err_msg)).into_response()
            }
            InsufficientAuthorizedStorage(_, _) => {
                let err_msg = serde_json::json!({ "msg": format!("{self}") });
                (StatusCode::PAYLOAD_TOO_LARGE, Json(err_msg)).into_response()
            }
        }
    }
}
