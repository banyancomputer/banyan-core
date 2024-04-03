use axum::extract::{Json, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use jwt_simple::prelude::*;
use serde::Deserialize;
use time::OffsetDateTime;
use validify::{Validate, Validify};

use crate::api::models::ApiBucketAccess;
use crate::app::AppState;
use crate::database::models::{Bucket, BucketAccessState, BucketType, StorageClass, UserKey};
use crate::extractors::ApiIdentity;

pub async fn handler(
    api_id: ApiIdentity,
    State(state): State<AppState>,
    Json(request): Json<CreateBucketRequest>,
) -> Result<Response, CreateBucketError> {
    request.validate()?;
    let database = state.database();
    let now = OffsetDateTime::now_utc();

    let mut conn = database.acquire().await?;
    let user_key = UserKey::by_fingerprint(&mut conn, api_id.key_fingerprint()).await?;
    if !user_key.api_access || user_key.user_id != api_id.user_id().to_string() {
        return Err(CreateBucketError::Unauthorized);
    }
    conn.close().await?;

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

    let mut conn = database.acquire().await?;
    let access = Bucket::set_access(
        &mut conn,
        &user_key.id,
        &bucket_id,
        BucketAccessState::Approved,
    )
    .await
    .map_err(CreateBucketError::GrantAccessFailed)?;

    let bucket = Bucket::find_by_id(&mut conn, &bucket_id)
        .await
        .map_err(CreateBucketError::BucketLookupFailed)?;

    let resp = ApiCreateBucketResponse {
        id: bucket.id,
        name: bucket.name,
        r#type: bucket.r#type,
        storage_class: bucket.storage_class,
        access: ApiBucketAccess {
            fingerprint: user_key.fingerprint,
            state: access.state,
        },
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
    access: ApiBucketAccess,
}

#[derive(Debug, thiserror::Error)]
pub enum CreateBucketError {
    #[error("key is unauthorized for API use")]
    Unauthorized,

    #[error("retrieving additional bucket details failed: {0}")]
    Database(#[from] sqlx::Error),

    #[error("retrieving additional bucket details failed: {0}")]
    BucketLookupFailed(sqlx::Error),

    #[error("failed to insert bucket into database: {0}")]
    BucketCreationFailed(sqlx::Error),

    #[error("failed to insert bucket key into database: {0}")]
    GrantAccessFailed(sqlx::Error),

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
    impl UserKey {
        pub async fn find_by_id(
            conn: &mut DatabaseConnection,
            id: &str,
        ) -> Result<UserKey, sqlx::Error> {
            sqlx::query_as!(UserKey, "SELECT * FROM user_keys WHERE id = $1;", id)
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
        /*

        let bucket_key = UserKey::find_by_id(&mut conn, &bucket_response.initial_bucket_key.id)
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
        */
    }
}
