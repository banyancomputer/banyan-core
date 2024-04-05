use axum::extract::{Path, State};
use axum::response::{IntoResponse, Response};
use axum::Json;
use http::StatusCode;

use crate::api::admin::users::all_users::RESETTABLE_USERS;
use crate::app::AppState;
use crate::database::models::User;
use crate::extractors::{AdminIdentity, ADMIN_USERS};

pub async fn handler(
    _: AdminIdentity,
    State(state): State<AppState>,
    Path(user_id): Path<String>,
) -> Result<Response, ResetUserError> {
    let database = state.database();
    let mut conn = database.acquire().await?;
    let user = User::find_by_id(&mut conn, &user_id)
        .await?
        .ok_or(ResetUserError::UserNotFound)?;

    if !ADMIN_USERS.contains(&user.email.as_str())
        && !RESETTABLE_USERS.contains(&user.email.as_str())
    {
        return Err(ResetUserError::NotResettable);
    }

    sqlx::query!("DELETE FROM sessions WHERE user_id = $1;", user_id)
        .execute(&mut *conn)
        .await
        .map_err(ResetUserError::DatabaseFailure)?;

    sqlx::query!(
        "DELETE FROM snapshot_restore_requests WHERE user_id = $1;",
        user_id
    )
    .execute(&mut *conn)
    .await
    .map_err(ResetUserError::DatabaseFailure)?;

    sqlx::query!(
        "DELETE FROM storage_hosts_metadatas_storage_grants
    WHERE storage_grant_id IN (SELECT id
                               FROM storage_grants
                               WHERE user_id = $1);",
        user_id
    )
    .execute(&mut *conn)
    .await
    .map_err(ResetUserError::DatabaseFailure)?;

    sqlx::query!(
        "DELETE FROM block_locations where metadata_id in (
        SELECT metadata_id
    FROM metadata
    JOIN buckets
    ON metadata.bucket_id = buckets.id
    WHERE user_id = $1);",
        user_id
    )
    .execute(&mut *conn)
    .await
    .map_err(ResetUserError::DatabaseFailure)?;

    sqlx::query!("DELETE FROM storage_grants WHERE user_id = $1;", user_id)
        .execute(&mut *conn)
        .await
        .map_err(ResetUserError::DatabaseFailure)?;

    sqlx::query!(r#"DELETE FROM users WHERE id = $1;"#, user_id)
        .execute(&mut *conn)
        .await
        .map_err(ResetUserError::DatabaseFailure)?;

    Ok((StatusCode::OK, Json("User reset successfully")).into_response())
}

#[derive(Debug, thiserror::Error)]
pub enum ResetUserError {
    #[error("failed to query the database: {0}")]
    DatabaseFailure(#[from] sqlx::Error),
    #[error("user not found")]
    UserNotFound,
    #[error("user not resettable")]
    NotResettable,
}

impl IntoResponse for ResetUserError {
    fn into_response(self) -> Response {
        match &self {
            ResetUserError::DatabaseFailure(e) => {
                tracing::error!("error from database: {e}");
                let err_msg = serde_json::json!({"msg": "backend service experienced an issue servicing the request"});
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
            }
            ResetUserError::UserNotFound | ResetUserError::NotResettable => {
                let err_msg = serde_json::json!({ "msg": self.to_string() });
                (StatusCode::BAD_REQUEST, Json(err_msg)).into_response()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use axum::extract::Path;

    use crate::api::admin::users::all_users::RESETTABLE_USERS;
    use crate::api::admin::users::reset_user::handler;
    use crate::app::mock_app_state;
    use crate::database::models::MetadataState;
    use crate::database::test_helpers::{
        create_storage_grant, create_storage_host, get_or_create_session, sample_blocks,
        sample_bucket, sample_metadata, sample_user, setup_database,
    };
    use crate::extractors::AdminIdentity;

    #[tokio::test]
    async fn test_reset_user() {
        let db = setup_database().await;
        let mut conn = db.acquire().await.expect("connection");

        let user_id = sample_user(&mut conn, RESETTABLE_USERS[0]).await;
        let bucket_id = sample_bucket(&mut conn, &user_id).await;
        let metadata_id = sample_metadata(&mut conn, &bucket_id, 1, MetadataState::Current).await;
        let storage_host_id = create_storage_host(
            &mut conn,
            "random-host",
            "https://127.0.0.1:8001/",
            1_000_000,
        )
        .await;
        let storage_grant_id =
            create_storage_grant(&mut conn, &storage_host_id, &user_id, 100).await;
        sample_blocks(
            &mut conn,
            10,
            &metadata_id,
            &storage_host_id,
            &storage_grant_id,
        )
        .await;

        let result = handler(
            AdminIdentity::new(get_or_create_session(&mut conn, &user_id).await),
            mock_app_state(db.clone()),
            Path(user_id.clone()),
        )
        .await;

        assert!(result.is_ok());
        let response = result.unwrap();
        let status = response.status();
        assert_eq!(status, 200);
    }
}
