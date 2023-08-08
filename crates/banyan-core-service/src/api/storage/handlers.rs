use axum::extract::{self, Path};

use crate::api::buckets::{models, requests, responses};
use crate::extractors::{ApiToken, DbConn};

pub async fn create_authorization(
    _api_token: ApiToken,
    mut _db_conn: DbConn,
    extract::Json(new_auth): extract::Json<requests::CreateStorageAuthorization>,
) -> Response {
    if let Err(errors) = new_auth.validate() {
        return (
            StatusCode::BAD_REQUEST,
            format!("errors: {:?}", errors.errors()),
        )
            .into_response();
    }

    //(StatusCode::OK, axum::Json(response)).into_response()
    (StatusCode::OK, "todo").into_response()
}

pub async fn current_authorizations(
    _api_token: ApiToken,
    mut _db_conn: DbConn,
) -> Response {
    (StatusCode::OK, "todo").into_response()
}

pub async fn revoke_authorization(
    _api_token: ApiToken,
    Path(_auth_id): Path<Uuid>,
    mut _db_conn: DbConn,
) -> Response {
    (StatusCode::OK, "todo").into_response()
}
