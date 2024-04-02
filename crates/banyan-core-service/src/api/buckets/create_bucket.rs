use axum::extract::{Json, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use jwt_simple::prelude::*;
use serde::Deserialize;
use time::OffsetDateTime;
use validify::{Validate, Validify};

use crate::app::AppState;
use crate::database::models::{ApiKey, Bucket, BucketAccessState, BucketType, StorageClass};
use crate::extractors::ApiIdentity;
use crate::utils::keys::fingerprint_public_key;

pub async fn handler(
    api_id: ApiIdentity,
    State(state): State<AppState>,
    Json(request): Json<CreateBucketRequest>,
) -> Result<Response, CreateBucketError> {
    request.validate()?;
    let database = state.database();
    let now = OffsetDateTime::now_utc();

    let user_id = api_id.user_id().to_string();
    let bucket_id = sqlx::query_scalar!(
        r#"
            INSERT INTO buckets (user_id, name, type, storage_class, updated_at)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id;
        "#,
        user_id,
        request.name,
        request.bucket_type,
        request.storage_class,
        now,
    )
    .fetch_one(&database)
    .await
    .map_err(CreateBucketError::BucketCreationFailed)?;

    // todo: when the extra returns have been removed this can turn into an execute query, for now
    // we need to keep a handle on the id
    //let
    // Provide this Api Key with Bucket Access

    Bucket::set_access(
        &database,
        &user_key_id,
        &bucket_id,
        BucketAccessState::Approved,
    )
    .await?;
    let bucket_key_id = sqlx::query_scalar!(
        r#"
            INSERT INTO bucket_access (api_key_id, bucket_id, state)
            VALUES ($1, $2, 'approved')
            RETURNING id;
        "#,
        api_key_id,
        bucket_id,
    )
    .fetch_one(&database)
    .await
    .map_err(CreateBucketError::BucketKeyCreationFailed)?;
    let mut conn = database
        .acquire()
        .await
        .map_err(CreateBucketError::BucketKeyCreationFailed)?;
    let bucket = Bucket::find_by_id(&mut conn, &bucket_id)
        .await
        .map_err(CreateBucketError::BucketKeyCreationFailed)?;

    let bucket_key = sqlx::query_as!(
        ApiKey,
        "SELECT * FROM api_bucket_keys WHERE id = $1;",
        bucket_key_id
    )
    .fetch_one(&mut *conn)
    .await
    .map_err(CreateBucketError::AdditionalDetailsUnavailable)?;

    let resp = ApiCreateBucketResponse {
        id: bucket.id,
        name: bucket.name,
        r#type: bucket.r#type,
        storage_class: bucket.storage_class,
        state: bucket_access.state,
    };

    Ok((StatusCode::OK, Json(resp)).into_response())
}

#[derive(Clone, Debug, Deserialize, Validify)]
pub struct CreateBucketRequest {
    #[validate(length(min = 3, max = 32))]
    name: String,

    #[serde(rename = "type")]
    bucket_type: BucketType,
    storage_class: StorageClass,
}

#[derive(Debug, Serialize, Deserialize)]
struct ApiCreateBucketResponse {
    id: String,
    name: String,
    r#type: BucketType,
    storage_class: StorageClass,
    state: BucketAccessState,
}

#[derive(Debug, thiserror::Error)]
pub enum CreateBucketError {
    #[error("retrieving additional bucket details failed: {0}")]
    AdditionalDetailsUnavailable(sqlx::Error),

    #[error("failed to insert bucket into database: {0}")]
    BucketCreationFailed(sqlx::Error),

    #[error("failed to insert bucket key into database: {0}")]
    BucketKeyCreationFailed(sqlx::Error),

    #[error("invalid bucket creation request received: {0}")]
    InvalidBucket(#[from] validify::ValidationErrors),

    #[error("provided public key was not valid: {0}")]
    InvalidPublicKey(jwt_simple::Error),
}

impl IntoResponse for CreateBucketError {
    fn into_response(self) -> Response {
        use CreateBucketError as CBE;

        match self {
            CBE::InvalidBucket(_) | CBE::InvalidPublicKey(_) => {
                let err_msg = serde_json::json!({"msg": "{self}"});
                (StatusCode::BAD_REQUEST, Json(err_msg)).into_response()
            }
            _ => {
                tracing::error!("{self}");
                let err_msg = serde_json::json!({"msg": "backend service experienced an issue servicing the request"});
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::buckets::create_bucket::CreateBucketRequest;
    use crate::app::mock_app_state;
    use crate::database::test_helpers::{get_or_create_identity, sample_user, setup_database};
    use crate::database::DatabaseConnection;
    use crate::utils::tests::deserialize_response;
    impl ApiKey {
        pub async fn find_by_id(
            conn: &mut DatabaseConnection,
            id: &str,
        ) -> Result<ApiKey, sqlx::Error> {
            sqlx::query_as!(ApiKey, "SELECT * FROM api_keys WHERE id = $1;", id)
                .fetch_one(conn)
                .await
        }
    }

    #[tokio::test]
    async fn test_create_bucket() {
        let db = setup_database().await;
        let mut conn = db.acquire().await.expect("connection");

        let user_id = sample_user(&mut conn, "test@example.com").await;
        let key_pair = ES384KeyPair::generate();

        let new_config = CreateBucketRequest {
            name: "new_name".to_string(),
            bucket_type: BucketType::Backup,
            storage_class: StorageClass::Hot,
            initial_bucket_key_pem: key_pair.public_key().to_pem().expect("pem"),
        };

        let result = handler(
            get_or_create_identity(&mut conn, &user_id).await,
            mock_app_state(db.clone()),
            Json(new_config.clone()),
        )
        .await;

        assert!(result.is_ok());
        let response = result.unwrap();
        let status = response.status();
        let bucket_response: ApiCreateBucketResponse = deserialize_response(response).await;
        let bucket_in_db = Bucket::find_by_id(&mut conn, &bucket_response.id)
            .await
            .unwrap();
        let bucket_key = ApiKey::find_by_id(&mut conn, &bucket_response.initial_bucket_key.id)
            .await
            .unwrap();
        assert_eq!(status, StatusCode::OK);
        assert_eq!(bucket_response.id, bucket_in_db.id);
        assert_eq!(user_id, bucket_in_db.user_id);
        assert_eq!(bucket_response.r#type, bucket_in_db.r#type);
        assert_eq!(bucket_response.storage_class, bucket_in_db.storage_class);
        assert_eq!(bucket_key.id, bucket_response.initial_bucket_key.id);
        assert_eq!(
            bucket_key.fingerprint,
            bucket_response.initial_bucket_key.fingerprint
        );
        //assert_eq!(bucket_key.state, bucket_response.initial_bucket_key.state);
    }
}
