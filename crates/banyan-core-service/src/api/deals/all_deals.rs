use axum::extract::{Query, State};
use axum::response::{IntoResponse, Response};
use axum::Json;
use http::StatusCode;

use crate::api::models::ApiDeal;
use crate::app::AppState;
use crate::database::models::{Deal, DealState, DealStateError};
use crate::extractors::StorageProviderIdentity;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct DealQuery {
    pub status: Option<String>,
}

pub async fn handler(
    _: StorageProviderIdentity,
    State(state): State<AppState>,
    Query(query): Query<DealQuery>,
) -> Result<Response, AllDealsError> {
    let database = state.database();
    let status = match &query.status {
        Some(status) => DealState::try_from(status.as_str()).map_err(AllDealsError::QueryError)?,
        None => DealState::Active,
    };
    let query_result = sqlx::query_as!(
        Deal,
        r#"SELECT d.id, d.state, SUM(ss.size) AS size, accepted_by, accepted_at
        FROM deals d
            JOIN snapshot_segments ss ON d.id = ss.deal_id
        WHERE d.state = $1
        GROUP BY d.id;"#,
        status
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
    #[error("query error: {0}")]
    QueryError(DealStateError),
}

impl IntoResponse for AllDealsError {
    fn into_response(self) -> Response {
        match self {
            AllDealsError::QueryError(err) => {
                tracing::error!("Bad request on looking up all deals: {err}");
                let err_msg = serde_json::json!({"msg": "Invalid deal status"});
                (StatusCode::BAD_REQUEST, Json(err_msg)).into_response()
            }
            _ => {
                tracing::error!("Internal server error on looking up all deals: {self}");
                let err_msg = serde_json::json!({"msg": "backend service experienced an issue servicing the request"});
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use axum::extract::Query;

    use crate::api::deals::all_deals::{handler, DealQuery};
    use crate::api::models::ApiDeal;
    use crate::app::mock_app_state;
    use crate::database::models::DealState;
    use crate::database::{test_helpers, DatabaseConnection};
    use crate::extractors::StorageProviderIdentity;
    use crate::tasks::BLOCK_SIZE;
    use crate::utils::tests::deserialize_result;

    async fn setup_deals(db: &mut DatabaseConnection) -> Result<Vec<String>, sqlx::Error> {
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
            let deal_id = test_helpers::create_deal(db, deal_state, None, None)
                .await
                .unwrap();
            deal_ids.push(deal_id);
        }

        Ok(deal_ids)
    }

    #[tokio::test]
    async fn test_insert_and_retrieve_all_deals() {
        let db = test_helpers::setup_database().await;
        let mut conn = db.acquire().await.expect("connection");

        let created_deals = setup_deals(&mut conn).await.unwrap();

        let res = handler(
            StorageProviderIdentity::default(),
            mock_app_state(db.clone()),
            Query(DealQuery { status: None }),
        )
        .await;

        let deals: Vec<ApiDeal> = deserialize_result(res).await;
        assert_eq!(deals.len(), 2);
        assert!(deals.iter().all(|deal| deal.state == DealState::Active));
        // because of the number of blocks in create_deal() not because there are two deals
        assert!(deals.iter().all(|deal| deal.size == 2 * BLOCK_SIZE));
        assert!(deals.iter().all(|deal| created_deals.contains(&deal.id)));
    }
}
