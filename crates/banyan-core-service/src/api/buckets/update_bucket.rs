use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use uuid::Uuid;

use crate::api::models::ApiBucketConfiguration;
use crate::app::AppState;
use crate::database::models::Bucket;
use crate::extractors::UserIdentity;

pub async fn handler(
    user_identity: UserIdentity,
    State(state): State<AppState>,
    Path(bucket_id): Path<Uuid>,
    Json(request): Json<ApiBucketConfiguration>,
) -> Result<Response, BucketUsageError> {
    let bucket_id = bucket_id.to_string();
    let user_id = user_identity.id().to_string();
    let database = state.database();
    let mut conn = database.acquire().await?;
    let bucket = Bucket::find_by_id(&mut conn, &bucket_id).await?;
    if bucket.user_id != user_id || bucket.deleted_at.is_some() {
        return Err(BucketUsageError::NotFound);
    }
    if request.replicas.is_some() && request.replicas.unwrap() < bucket.replicas {
        return Err(BucketUsageError::ReplicasCannotBeReduced);
    }

    Bucket::update_configuration(&mut conn, &bucket_id, &request).await?;
    Ok((StatusCode::OK, Json(())).into_response())
}

#[derive(Debug, thiserror::Error)]
pub enum BucketUsageError {
    #[error("failed to run query: {0}")]
    QueryFailure(#[from] sqlx::Error),
    #[error("invalid bucket configuration: {0}")]
    InvalidBucket(#[from] validify::ValidationErrors),
    #[error("not found")]
    NotFound,
    #[error("replicas cannot be reduced")]
    ReplicasCannotBeReduced,
}

impl IntoResponse for BucketUsageError {
    fn into_response(self) -> Response {
        let (status_code, err_msg) = match self {
            BucketUsageError::NotFound => (StatusCode::NOT_FOUND, "not found"),
            BucketUsageError::ReplicasCannotBeReduced => {
                (StatusCode::BAD_REQUEST, "replicas cannot be reduced")
            }
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "a backend service issue encountered an error",
            ),
        };
        tracing::error!("internal error handling bucket usage request: {self}");
        let err_msg = serde_json::json!({"msg": err_msg});
        (status_code, Json(err_msg)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use axum::extract::{Json, Path};
    use uuid::Uuid;

    use crate::api::buckets::update_bucket::handler;
    use crate::api::models::ApiBucketConfiguration;
    use crate::app::mock_app_state;
    use crate::database::models::Bucket;
    use crate::database::test_helpers::{
        get_or_create_session, sample_bucket, sample_user, setup_database,
    };
    use crate::extractors::UserIdentity;

    #[tokio::test]
    async fn test_bucket_configuration_update() {
        let db = setup_database().await;
        let mut conn = db.acquire().await.expect("connection");

        let user_id = sample_user(&mut conn, "test@example.com").await;
        let bucket_id = sample_bucket(&mut conn, &user_id).await;

        let new_config = ApiBucketConfiguration {
            name: Some("new_name".to_string()),
            replicas: Some(3),
        };

        let res = handler(
            UserIdentity::Session(get_or_create_session(&mut conn, &user_id).await),
            mock_app_state(db.clone()),
            Path(Uuid::parse_str(&bucket_id).expect("bucket id as uuid")),
            Json(new_config.clone()),
        )
        .await;

        assert!(res.is_ok());
        let updated_bucket = Bucket::find_by_id(&mut conn, &bucket_id).await.unwrap();
        assert_eq!(updated_bucket.name, new_config.name.unwrap());
        assert_eq!(updated_bucket.replicas, new_config.replicas.unwrap());
    }
}
