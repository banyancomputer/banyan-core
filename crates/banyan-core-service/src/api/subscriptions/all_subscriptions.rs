use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use crate::api::models::ApiSubscription;
use crate::app::AppState;
use crate::database::models::{Bucket, Subscription};
use crate::extractors::UserIdentity;

pub async fn handler(user: Option<UserIdentity>, State(state): State<AppState>) -> Response {
    let database = state.database();

    let mut conn = match database.acquire().await {
        Ok(conn) => conn,
        Err(err) => {
            tracing::error!("failed to acquire database connection: {err}");
            let err_msg = serde_json::json!({"msg": "backend service experienced an issue servicing the request"});
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
        },
    };

    let current_subscription_id = match user {
        Some(u) => {
            let user_id = u.id().to_string();

            let q_res = sqlx::query_scalar!(
                "SELECT subscription_id FROM users WHERE id = $1;",
                user_id,
            )
            .fetch_one(&mut *conn)
            .await;

            match q_res {
                Ok(sid) => Some(sid),
                Err(err) => {
                    tracing::error!("failed to lookup user's subscription: {err}");
                    let err_msg = serde_json::json!({"msg": "backend service experienced an issue servicing the request"});
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
                }
            }
        },
        None => None,
    };

    todo!()
}
