use axum::extract::{Json, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use crate::api::models::ApiUser;
use crate::app::AppState;
use crate::database::models::User;
use crate::extractors::UserIdentity;

pub async fn handler(user_identity: UserIdentity, State(state): State<AppState>) -> Response {
    let database = state.database();
    let mut conn = match database.acquire().await {
        Ok(conn) => conn,
        Err(err) => {
            tracing::error!("failed to acquire database connection: {err}");
            let err_msg = serde_json::json!({"msg": "backend service experienced an issue servicing the request"});
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response();
        }
    };

    let user_id = user_identity.id().to_string();

    match User::find_by_id(&mut conn, &user_id).await {
        Ok(Some(u)) => (StatusCode::OK, Json(ApiUser::from(u))).into_response(),
        Ok(None) => {
            let err_msg = serde_json::json!({"msg": "not found"});
            (StatusCode::NOT_FOUND, Json(err_msg)).into_response()
        }
        Err(err) => {
            tracing::error!("failed to lookup user: {err}");
            let err_msg = serde_json::json!({"msg": "backend service experienced an issue servicing the request"});
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
        }
    }
}
