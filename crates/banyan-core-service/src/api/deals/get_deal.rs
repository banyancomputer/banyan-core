use axum::extract::{Path, State};
use axum::response::{IntoResponse, Response};
use axum::Json;
use http::StatusCode;
use uuid::Uuid;

use crate::api::models::ApiDeal;
use crate::app::AppState;
use crate::database::models::{Deal, DealState};
use crate::extractors::StorageProviderIdentity;

pub async fn handler(
    storage_provider: StorageProviderIdentity,
    State(state): State<AppState>,
    Path(deal_id): Path<Uuid>,
) -> Response {
    let database = state.database();
    let deal_id = deal_id.to_string();
    let active = DealState::Active.to_string();
    let query_result = sqlx::query_as!(
        Deal,
        r#"SELECT * from deals WHERE id = $1 AND (state=$2 OR accepted_by=$3);"#,
        deal_id,
        active,
        storage_provider.id
    )
    .fetch_one(&database)
    .await;

    match query_result {
        Ok(b) => (StatusCode::OK, Json(ApiDeal::from(b))).into_response(),
        Err(sqlx::Error::RowNotFound) => {
            let err_msg = serde_json::json!({"msg": "not found"});
            (StatusCode::NOT_FOUND, Json(err_msg)).into_response()
        }
        Err(err) => {
            tracing::error!("failed to lookup specific bucket for account: {err}");
            let err_msg = serde_json::json!({"msg": "backend service experienced an issue servicing the request"});
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use axum::extract::Path;
    use uuid::Uuid;

    use super::*;
    use crate::database::models::DealState;
    use crate::database::test_helpers;
    use crate::utils::tests::mock_app_state;

    #[tokio::test]
    async fn test_get_deal_active() {
        let db = test_helpers::setup_database().await;
        let active_deal_id = test_helpers::create_deal(&db, DealState::Active, None)
            .await
            .unwrap();

        let res = handler(
            StorageProviderIdentity {
                id: "test_host_id".to_string(),
            },
            mock_app_state(db.clone()),
            Path(Uuid::parse_str(active_deal_id.as_str()).unwrap()),
        )
        .await;

        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_owned_accepted_deals_returned() {
        let db = test_helpers::setup_database().await;
        let host_id = test_helpers::create_storage_hosts(&db, "http://mock.com", "mock_name")
            .await
            .unwrap();
        let accepted_deal_id =
            test_helpers::create_deal(&db, DealState::Accepted, Some(host_id.clone()))
                .await
                .unwrap();

        let res = handler(
            StorageProviderIdentity {
                id: host_id.clone(),
            },
            mock_app_state(db.clone()),
            Path(Uuid::parse_str(accepted_deal_id.as_str()).unwrap()),
        )
        .await;

        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_not_owned_accepted_deals_not_returned() {
        let db = test_helpers::setup_database().await;
        let accepted_deal_id = test_helpers::create_deal(&db, DealState::Accepted, None)
            .await
            .unwrap();

        let res = handler(
            StorageProviderIdentity {
                id: "test_host_id".to_string(),
            },
            mock_app_state(db.clone()),
            Path(Uuid::parse_str(accepted_deal_id.as_str()).unwrap()),
        )
        .await;

        assert_eq!(res.status(), StatusCode::NOT_FOUND);
    }
}