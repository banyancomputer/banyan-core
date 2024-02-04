use axum::extract::{Path, State};
use axum::response::{IntoResponse, Response};
use axum::Json;
use http::StatusCode;
use uuid::Uuid;

use crate::app::AppState;
use crate::database::models::DealState;
use crate::extractors::StorageProviderIdentity;

pub async fn handler(
    storage_provider: StorageProviderIdentity,
    State(state): State<AppState>,
    Path(deal_id): Path<Uuid>,
) -> Response {
    let database = state.database();
    let deal_id = deal_id.to_string();

    let query_result = sqlx::query!(
        r#"UPDATE deals SET state =$1, accepted_by=$2, accepted_at=DATETIME('now') WHERE id = $3  AND state == $4;"#,
        DealState::Accepted,
        storage_provider.id,
        deal_id,
        DealState::Active
    )
        .execute(&database)
        .await;

    match query_result {
        Ok(result) => {
            if result.rows_affected() == 0 {
                let err_msg = serde_json::json!({"msg": "not found"});
                (StatusCode::NOT_FOUND, Json(err_msg)).into_response()
            } else {
                (StatusCode::NO_CONTENT, ()).into_response()
            }
        }
        Err(err) => {
            tracing::error!("failed to update deal: {err}");
            let err_msg = serde_json::json!({"msg": "a backend service issue occurred"});
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use axum::extract::Path;
    use http::StatusCode;
    use uuid::Uuid;

    use crate::api::deals::accept_deal::handler;
    use crate::app::mock_app_state;
    use crate::database::models::DealState;
    use crate::database::test_helpers;
    use crate::extractors::StorageProviderIdentity;

    #[tokio::test]
    async fn test_accept_deal() {
        let db = test_helpers::setup_database().await;
        let mut conn = db.acquire().await.expect("connection");

        let host_id = test_helpers::create_storage_hosts(&mut conn, "http://mock.com", "mock_name")
            .await
            .unwrap();
        let active_deal_id = test_helpers::create_deal(&mut conn, DealState::Active, None, None)
            .await
            .unwrap();
        test_helpers::create_deal(&mut conn, DealState::Accepted, None, None)
            .await
            .unwrap();

        let res = handler(
            StorageProviderIdentity {
                id: host_id,
                name: String::from("mock_name"),
            },
            mock_app_state(db.clone()),
            Path(Uuid::parse_str(active_deal_id.as_str()).unwrap()),
        )
        .await;

        let status_code = res.status();
        assert_eq!(status_code, StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn test_accepted_deals_cannot_be_accepted_again() {
        let db = test_helpers::setup_database().await;
        let mut conn = db.acquire().await.expect("success");

        let host_id =
            test_helpers::create_storage_hosts(&mut conn, "http://mock.com", "mock_name").await;
        let accepted_deal_id =
            test_helpers::create_deal(&mut conn, DealState::Accepted, None, None)
                .await
                .unwrap();

        let res = handler(
            StorageProviderIdentity {
                id: host_id.unwrap(),
                name: String::from("mock_name"),
            },
            mock_app_state(db.clone()),
            Path(Uuid::parse_str(accepted_deal_id.as_str()).unwrap()),
        )
        .await;

        let status_code = res.status();
        assert_eq!(status_code, StatusCode::NOT_FOUND);
    }
}
