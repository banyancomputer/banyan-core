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
    let query_result = sqlx::query_as!(
        Deal,
        r#"SELECT d.id, d.state,  COALESCE(SUM(ss.size), 0) AS size, accepted_by, accepted_at
            FROM deals d
                JOIN snapshot_segments ss ON d.id = ss.deal_id
            WHERE d.id = $1 AND (d.state=$2 OR d.accepted_by=$3)
            GROUP BY d.id;"#,
        deal_id,
        DealState::Active,
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
    use crate::app::mock_app_state;
    use crate::database::models::DealState;
    use crate::database::test_helpers;
    use crate::tasks::BLOCK_SIZE;
    use crate::utils::tests::deserialize_response;

    #[tokio::test]
    async fn test_get_active_deal() {
        let db = test_helpers::setup_database().await;
        let mut conn = db.acquire().await.expect("connection");

        let active_deal_id = test_helpers::create_deal(&mut conn, DealState::Active, None, None)
            .await
            .unwrap();

        let res = handler(
            StorageProviderIdentity {
                id: "test_host_id".to_string(),
                name: "mock_name".to_string(),
            },
            mock_app_state(db.clone()),
            Path(Uuid::parse_str(active_deal_id.as_str()).unwrap()),
        )
        .await;

        let status = res.status();
        let deal: ApiDeal = deserialize_response(res).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(deal.id, active_deal_id);
        assert_eq!(deal.state, DealState::Active);
        // hardcoded in segment creation
        assert_eq!(deal.size, 2 * BLOCK_SIZE);
    }

    #[tokio::test]
    async fn test_owned_accepted_deal_is_returned() {
        let db = test_helpers::setup_database().await;
        let mut conn = db.acquire().await.expect("connection");

        let host_id =
            test_helpers::create_storage_hosts(&mut conn, "http://mock.com", "mock_name").await;
        let accepted_deal_id =
            test_helpers::create_deal(&mut conn, DealState::Accepted, None, Some(host_id.clone()))
                .await
                .unwrap();

        let res = handler(
            StorageProviderIdentity {
                id: host_id.clone(),
                name: "mock_name".to_string(),
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
        let mut conn = db.acquire().await.expect("connection");

        let accepted_deal_id =
            test_helpers::create_deal(&mut conn, DealState::Accepted, None, None)
                .await
                .unwrap();

        let res = handler(
            StorageProviderIdentity {
                id: "test_host_id".to_string(),
                name: "mock_name".to_string(),
            },
            mock_app_state(db.clone()),
            Path(Uuid::parse_str(accepted_deal_id.as_str()).unwrap()),
        )
        .await;

        assert_eq!(res.status(), StatusCode::NOT_FOUND);
    }
}
