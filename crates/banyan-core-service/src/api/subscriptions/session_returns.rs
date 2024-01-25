use axum::extract::{Path, State};
use axum::response::{IntoResponse, Redirect, Response};

use crate::app::AppState;
use crate::database::models::StripeCheckoutSession;
use crate::extractors::UserIdentity;

pub async fn cancel_redirect() -> Response {
    Redirect::to("/").into_response()
}

pub async fn success_redirect(
    user_id: UserIdentity,
    State(state): State<AppState>,
    Path(checkout_session_id): Path<String>,
) -> Response {
    let redirect = Redirect::to("/").into_response();

    let database = state.database();
    let mut conn = match database.acquire().await {
        Ok(c) => c,
        Err(_) => return redirect,
    };

    let user_id = user_id.id().to_string();

    let mut checkout_session =
        match StripeCheckoutSession::find_by_stripe_id(&mut *conn, &user_id, &checkout_session_id)
            .await
        {
            Ok(Some(cs)) => cs,
            _ => return redirect,
        };

    let _ = checkout_session.complete(&mut *conn).await;

    redirect
}
