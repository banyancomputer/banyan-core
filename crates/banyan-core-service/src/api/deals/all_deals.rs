use axum::extract::State;
use axum::response::{IntoResponse, Response};
use axum::Json;
use http::StatusCode;

use crate::api::models::ApiDeal;
use crate::app::AppState;
use crate::database::models::{Deal, DealState};
use crate::extractors::StorageProviderIdentity;

pub async fn handler(
    _: StorageProviderIdentity,
    State(state): State<AppState>,
) -> Result<Response, AllDealsError> {
    let database = state.database();
    let query_result = sqlx::query_as!(
        Deal,
        "SELECT d.id, d.state, m.data_size AS size
            FROM deals AS d
            JOIN main.snapshots s on d.id = s.deal_id
            JOIN main.metadata m on m.id = s.metadata_id
            WHERE d.state=$1;",
        DealState::Active
    )
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
    use crate::utils::tests::{deserialize_result, mock_app_state};

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
        let created_deals = setup_deals(&db).await.unwrap();

        let res = handler(
            StorageProviderIdentity {
                id: String::from("1"),
            },
            mock_app_state(db.clone()),
        )
        .await;

        let deals: Vec<ApiDeal> = deserialize_result(res).await;
        assert_eq!(deals.len(), 2);
        assert!(deals.iter().all(|deal| deal.state == DealState::Active));
        assert!(deals.iter().all(|deal| deal.size == 0));
        assert!(deals.iter().all(|deal| created_deals.contains(&deal.id)));
    }
}
