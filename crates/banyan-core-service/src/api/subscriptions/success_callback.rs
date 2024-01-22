use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect, Response};

use crate::app::AppState;
use crate::database::models::StripeCheckoutSession;
use crate::extractors::UserIdentity;

pub async fn handler(
    user_id: UserIdentity,
    State(state): State<AppState>,
    Path(checkout_session_id): Path<String>,
) -> Response {
    // checkouts are only allowed via sessions, but give API users a better message.
    let session_id = match &user_id {
        UserIdentity::Api(_) => {
            let err_msg = serde_json::json!({"msg": "subscription changes can only take place through session authentication"});
            return (StatusCode::BAD_REQUEST, Json(err_msg)).into_response();
        }
        UserIdentity::Session(sess) => sess.session_id().to_string(),
    };

    let database = state.database();
    let mut conn = match database.acquire().await {
        Ok(c) => c,
        Err(err) => {
            tracing::error!("error from database: {err}");
            return Redirect::temporary("/").into_response();
        }
    };

    if let Err(err) = StripeCheckoutSession::complete(&mut *conn, &session_id, &checkout_session_id).await {
        tracing::error!("error from database: {err}");
    }

    Redirect::temporary("/").into_response()
}
