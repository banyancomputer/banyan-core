use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use crate::extractors::{AuthenticatedClient, Database, UploadStore};

pub async fn handler(
    _db: Database,
    _client: AuthenticatedClient,
    _store: UploadStore
    TypedHeader(_content_type): TypedHeader<ContentType>,
    _body: BodyStream,
) -> Response {
    // parse request data
    // check content length against quota
    // record that an upload is in progress
    // collect upload in a temporary directory
    // during upload, if it goes over content length warn and start watching remaining authorized
    // storage
    // if upload exceeds authorized storage abort the upload with an unauthorized response
    // if upload errors clean up files and record the failure in the database with the uploaded amount
    // if upload succeeds queue task to report back to platform
    // report upload as success
    (StatusCode::NO_CONTENT, ()).into_response()
}
