use axum::body::StreamBody;
use axum::extract::{BodyStream, State, Path};
use axum::headers::ContentType;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use axum::TypedHeader;
use http::{HeaderMap, HeaderValue};
use object_store::ObjectStore;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

use crate::app::AppState;
use crate::database::Database;
use crate::extractors::{ApiIdentity, DataStore, SigningKey};
//use crate::utils::metadata_upload::{handle_metadata_upload, round_to_nearest_100_mib};
//use crate::utils::storage_ticket::generate_storage_ticket;

pub async fn handler(
    api_id: ApiIdentity,
    State(state): State<AppState>,
    store: DataStore,
    //signing_key: SigningKey,
    Path(bucket_id): Path<Uuid>,
    TypedHeader(content_type): TypedHeader<ContentType>,
    body: BodyStream,
) -> Response {
    todo!()
}
