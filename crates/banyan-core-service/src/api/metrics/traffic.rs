use axum::extract::{Json, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use time::error::ComponentRange;
use time::OffsetDateTime;

use crate::app::AppState;
use crate::extractors::StorageProviderIdentity;

pub async fn handler(
    _: StorageProviderIdentity,
    State(state): State<AppState>,
    Json(request): Json<MeterTrafficRequest>,
) -> Result<Response, MeterTrafficError> {
    let database = state.database();
    let storage_host_id: String = sqlx::query_scalar!(
        "SELECT id FROM storage_hosts WHERE name = $1",
        request.storage_host_name,
    )
    .fetch_one(&database)
    .await
    .map_err(MeterTrafficError::ClientNotFound)?;

    let created_at = match OffsetDateTime::from_unix_timestamp(request.slot) {
        Ok(created_at) => created_at,
        Err(err) => return Err(MeterTrafficError::TimestampParseError(err)),
    };

    sqlx::query!(
        r#"INSERT INTO metrics_traffic (user_id, ingress, egress,storage_host_id, slot)
           VALUES ($1, $2, $3, $4, $5)"#,
        request.user_id,
        request.ingress,
        request.egress,
        storage_host_id,
        created_at,
    )
    .execute(&database)
    .await
    .map_err(MeterTrafficError::FailedToStoreTrafficData)?;

    Ok((StatusCode::OK, ()).into_response())
}

#[derive(Debug, thiserror::Error)]
pub enum MeterTrafficError {
    #[error("failed to parse timetstamp: {0}")]
    TimestampParseError(ComponentRange),

    #[error("failed to store traffic data: {0}")]
    FailedToStoreTrafficData(sqlx::Error),

    #[error("client not found: {0}")]
    ClientNotFound(sqlx::Error),
}

impl IntoResponse for MeterTrafficError {
    fn into_response(self) -> Response {
        match self {
            MeterTrafficError::ClientNotFound(_) => (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"msg": "client not found"})),
            )
                .into_response(),
            MeterTrafficError::TimestampParseError(_) => (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"msg": "failed to parse timestamp"})),
            )
                .into_response(),
            MeterTrafficError::FailedToStoreTrafficData(_) => {
                tracing::error!("failed to store traffic data: {:#?}", self);
                (StatusCode::INTERNAL_SERVER_ERROR, Json( serde_json::json!({"msg": "backend service experienced an issue servicing the request"}))).into_response()
            }
        }
    }
}

#[derive(Deserialize, Clone)]
pub struct MeterTrafficRequest {
    pub user_id: String,
    pub storage_host_name: String,
    pub ingress: i64,
    pub egress: i64,
    pub slot: i64,
}

#[cfg(test)]
mod tests {
    use axum::extract::Json;
    use axum::http::StatusCode;
    use time::OffsetDateTime;

    use super::*;
    use crate::app::mock_app_state;
    use crate::database::test_helpers::{create_storage_hosts, sample_user, setup_database};

    fn setup_mock_request(user_id: &str, host_id: &str) -> MeterTrafficRequest {
        MeterTrafficRequest {
            user_id: user_id.to_string(),
            storage_host_name: host_id.to_string(),
            ingress: 100,
            egress: 200,
            slot: OffsetDateTime::now_utc().unix_timestamp(),
        }
    }

    #[tokio::test]
    async fn test_client_not_found_errors() {
        let db = setup_database().await;
        let state = mock_app_state(db.clone());
        let user_id = "fake_user";
        let host_id = "fake_host";

        let result = handler(
            StorageProviderIdentity {
                id: "test_host".to_string(),
            },
            state.clone(),
            Json(setup_mock_request(user_id, host_id).clone()),
        )
        .await;
        assert!(matches!(result, Err(MeterTrafficError::ClientNotFound(_))));
    }

    #[tokio::test]
    async fn test_missing_user_id_throws_error() {
        let db = setup_database().await;
        let state = mock_app_state(db.clone());
        let mut conn = db.begin().await.expect("connection");
        let storage_host_name = "mock_storage_host";
        create_storage_hosts(&mut conn, "http://mock.com", storage_host_name).await;
        conn.commit().await.expect("commit");
        let user_id = "fake_user";

        let result = handler(
            StorageProviderIdentity {
                id: "test_host".to_string(),
            },
            state.clone(),
            Json(setup_mock_request(user_id, storage_host_name).clone()),
        )
        .await;
        assert!(matches!(
            result,
            Err(MeterTrafficError::FailedToStoreTrafficData(_))
        ));
    }

    #[tokio::test]
    async fn same_slot_request_results_in_error() {
        let db = setup_database().await;
        let state = mock_app_state(db.clone());
        let mut conn = db.begin().await.expect("connection");
        let storage_host_name = "mock_storage_host";
        create_storage_hosts(&mut conn, "http://mock.com", storage_host_name).await;
        let user_id = sample_user(&mut conn, "user@domain.tld").await;
        conn.commit().await.expect("commit");
        let request = setup_mock_request(user_id.as_str(), storage_host_name).clone();

        let result = handler(
            StorageProviderIdentity {
                id: "test_host".to_string(),
            },
            state.clone(),
            Json(request.clone()),
        )
        .await;
        assert!(matches!(result, Ok(response) if response.status() == StatusCode::OK));
        let rows: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM metrics_traffic")
            .fetch_one(&state.database())
            .await
            .unwrap();
        assert_eq!(rows.0, 1);

        let result = handler(
            StorageProviderIdentity {
                id: "test_host".to_string(),
            },
            state.clone(),
            Json(request.clone()),
        )
        .await;
        assert!(matches!(
            result,
            Err(MeterTrafficError::FailedToStoreTrafficData(_))
        ));
    }
}
