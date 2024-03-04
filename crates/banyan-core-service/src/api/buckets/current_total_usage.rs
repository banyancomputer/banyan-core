use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;

use crate::app::AppState;
use crate::database::models::User;
use crate::extractors::UserIdentity;

pub async fn handler(
    user_identity: UserIdentity,
    State(state): State<AppState>,
) -> Result<Response, CurrentTotalUsageError> {
    let database = state.database();
    let mut conn = database.acquire().await?;

    // we need to include outdated currently as they include blocks referenced by the current
    // version, todo: we'll need a better way of calculating this
    let user_id = user_identity.id().to_string();
    let size = User::consumed_storage(&mut conn, &user_id).await?;

    let resp = serde_json::json!({"size": size});
    Ok((StatusCode::OK, Json(resp)).into_response())
}

#[derive(Debug, thiserror::Error)]
pub enum CurrentTotalUsageError {
    #[error("failed lookup: {0}")]
    Database(#[from] sqlx::Error),
}
impl IntoResponse for CurrentTotalUsageError {
    fn into_response(self) -> Response {
        tracing::error!("failed to lookup all deals: {self}");
        let err_msg = serde_json::json!({"msg": "backend service experienced an issue servicing the request"});
        (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use http::StatusCode;

    use crate::api::buckets::current_total_usage::handler;
    use crate::app::mock_app_state;
    use crate::database::models::MetadataState;
    use crate::database::test_helpers::{get_or_create_session, sample_bucket, sample_metadata};
    use crate::database::{test_helpers, DatabaseConnection};
    use crate::extractors::UserIdentity;

    pub async fn update_metadata_size(
        conn: &mut DatabaseConnection,
        metadata_id: &str,
        data_size: i64,
        metadata_size: i64,
    ) {
        sqlx::query!(
            "UPDATE metadata SET data_size = $1, metadata_size = $2 WHERE id = $3",
            data_size,
            metadata_size,
            metadata_id,
        )
        .execute(conn)
        .await
        .expect("update metadata size");
    }
    #[tokio::test]
    async fn test_handler_returns_zero_on_empty() {
        let db = test_helpers::setup_database().await;
        let mut conn = db.acquire().await.expect("connection");
        let user_id = test_helpers::sample_user(&mut conn, "user@domain.tld").await;

        let res = handler(
            UserIdentity::Session(get_or_create_session(&mut conn, &user_id).await),
            mock_app_state(db.clone()),
        )
        .await;
        assert!(res.is_ok());
        let response = res.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        assert_eq!(&body[..], b"{\"size\":0}");
    }

    #[tokio::test]
    async fn test_handler_returns_correct_size() {
        let db = test_helpers::setup_database().await;
        let mut conn = db.acquire().await.expect("connection");
        let user_id = test_helpers::sample_user(&mut conn, "user@domain.tld").await;
        let bucket_id = sample_bucket(&mut conn, &user_id).await;
        let metadata_id = sample_metadata(&mut conn, &bucket_id, 1, MetadataState::Current).await;
        let bucket_id_two = sample_bucket(&mut conn, &user_id).await;
        let metadata_id_two =
            sample_metadata(&mut conn, &bucket_id_two, 1, MetadataState::Current).await;
        update_metadata_size(&mut conn, &metadata_id, 100, 30).await;
        update_metadata_size(&mut conn, &metadata_id_two, 300, 50).await;
        let res = handler(
            UserIdentity::Session(get_or_create_session(&mut conn, &user_id).await),
            mock_app_state(db.clone()),
        )
        .await;
        assert!(res.is_ok());
        let response = res.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        assert_eq!(&body[..], b"{\"size\":480}");
    }
}
