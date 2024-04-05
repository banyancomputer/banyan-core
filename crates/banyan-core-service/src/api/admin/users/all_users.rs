use axum::extract::State;
use axum::response::{IntoResponse, Response};
use axum::Json;
use http::StatusCode;

use crate::api::models::ApiUsersAdmin;
use crate::app::AppState;
use crate::extractors::{AdminIdentity, ADMIN_USERS};

pub const RESETTABLE_USERS: [&str; 3] = [
    "ashley.stanhope@gmail.com",
    "juli@incana.org",
    "phristov@blocksoft.ltd",
];

pub async fn handler(
    _: AdminIdentity,
    State(state): State<AppState>,
) -> Result<Response, AllUsersError> {
    let database = state.database();
    let mut conn = database.acquire().await?;
    let resettable_users = Vec::from(RESETTABLE_USERS);
    let mut users = Vec::from(ADMIN_USERS);
    users.extend(resettable_users);

    let mut query_builder = sqlx::QueryBuilder::new(
        r#"SELECT id, email, verified_email, display_name, accepted_tos_at FROM users WHERE email IN ("#,
    );

    let mut separated_values = query_builder.separated(", ");
    for email in users {
        separated_values.push_bind(email);
    }
    query_builder.push(");");

    let users = query_builder
        .build_query_as::<ApiUsersAdmin>()
        .persistent(false)
        .fetch_all(&mut *conn)
        .await?;

    Ok((StatusCode::OK, Json(users)).into_response())
}

#[derive(Debug, thiserror::Error)]
pub enum AllUsersError {
    #[error("failed to query the database: {0}")]
    DatabaseFailure(#[from] sqlx::Error),
}

impl IntoResponse for AllUsersError {
    fn into_response(self) -> Response {
        tracing::error!("failed to lookup all users: {self}");
        let err_msg = serde_json::json!({"msg": "backend service experienced an issue servicing the request"});
        (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
    }
}
