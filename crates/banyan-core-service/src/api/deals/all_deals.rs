use axum::extract::State;
use axum::response::{IntoResponse, Response};
use axum::Json;
use http::StatusCode;

use crate::api::models::ApiDeal;
use crate::app::AppState;
use crate::database::models::{Deal, DealState};
use crate::extractors::StorageProviderIdentity;

pub async fn handler(
    _storage_provider: StorageProviderIdentity,
    State(state): State<AppState>,
) -> Result<Response, AllDealsError> {
    let database = state.database();
    let active_deals = DealState::Active.to_string();
    let query_result = sqlx::query_as!(Deal, "SELECT * FROM deals WHERE state=$1;", active_deals)
        .fetch_all(&database)
        .await
        .map_err(AllDealsError::DatabaseFailure)?;

    let deals: Vec<_> = query_result.into_iter().map(ApiDeal::from).collect();

    Ok((StatusCode::OK, Json(deals)).into_response())
}

#[derive(Debug, thiserror::Error)]
pub enum AllDealsError {
    #[error("failed to query the database: {0}")]
    DatabaseFailure(sqlx::Error),
}

impl IntoResponse for AllDealsError {
    fn into_response(self) -> Response {
        tracing::error!("failed to lookup all deals: {self}");
        let err_msg = serde_json::json!({"msg": "backend service experienced an issue servicing the request"});
        (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
    }
}
#[cfg(test)]
mod tests {
    use crate::api::deals::all_deals::handler;
    use crate::api::models::ApiDeal;
    use crate::database::models::DealState;
    use crate::database::{test_helpers, Database};
    use crate::extractors::StorageProviderIdentity;
    use crate::utils::tests::{deserialize_response, mock_app_state};

    async fn setup_deals(db: &Database) -> Result<Vec<String>, sqlx::Error> {
        let deal_states = vec![
            DealState::Active,
            DealState::Accepted,
            DealState::Active,
            DealState::Sealed,
            DealState::Finalized,
            DealState::Cancelled,
        ];
        let mut deal_ids = Vec::new();
        for deal_state in deal_states.into_iter() {
            let deal_id = test_helpers::create_deal(db, deal_state, None)
                .await
                .unwrap();
            deal_ids.push(deal_id);
        }
        Ok(deal_ids)
    }

    #[tokio::test]
    async fn test_insert_and_retrieve_all_deals() {
        let db = test_helpers::setup_database().await;
        let _deals = setup_deals(&db).await;

        let res = handler(
            StorageProviderIdentity {
                id: String::from("1"),
            },
            mock_app_state(db.clone()),
        )
        .await;

        let deals: Vec<ApiDeal> = deserialize_response(res).await;
        assert_eq!(deals.len(), 2);
        assert!(deals.iter().all(|deal| deal.state == DealState::Active));
    }
}
