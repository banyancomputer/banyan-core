use axum::extract::Json;
use axum::http::StatusCode;
use axum::response::IntoResponse;


use crate::api::auth::who_am_i::responses;
use crate::extractors::ApiToken;

/// Return the account id of the currently authenticated user
pub async fn read(api_token: ApiToken) -> impl IntoResponse {
    let response = responses::WhoAmI {
        account_id: api_token.subject(),
    };

    (StatusCode::OK, Json(response)).into_response()
}