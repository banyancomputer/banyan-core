use std::num::ParseIntError;
use std::ops::Add;

use axum::extract::{Query, State};
use axum::response::{IntoResponse, Response};
use axum::Json;
use http::StatusCode;
use serde::{Deserialize, Serialize};
use time::error::ComponentRange;
use time::{Duration, OffsetDateTime};

use crate::app::AppState;
use crate::database::models::{Deal, DealState, DealStateError};
use crate::extractors::StorageProviderIdentity;

const DEAL_SEAL_MAX_DELAY_DAYS: i64 = 3;
const PRICE_USD_PER_TB: f64 = 2.5;
const TB_IN_BYTES: f64 = 1_099_511_627_776.0;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct DealQuery {
    pub state: Option<String>,
}

pub async fn handler(
    identity: StorageProviderIdentity,
    State(state): State<AppState>,
    Query(query): Query<DealQuery>,
) -> Result<Response, AllDealsError> {
    let database = state.database();
    let state = match &query.state {
        Some(state) => DealState::try_from(state.as_str()).map_err(AllDealsError::QueryError)?,
        None => DealState::Active,
    };
    let mut query = sqlx::QueryBuilder::new(
        "
        SELECT d.id, d.state, SUM(ss.size) AS size, accepted_by, accepted_at
            FROM deals d
            JOIN snapshot_segments ss ON d.id = ss.deal_id
        WHERE d.state = ",
    );
    query.push_bind(state.to_string());
    if state != DealState::Active {
        query.push(" AND d.accepted_by = ");
        query.push_bind(identity.id);
    }
    query.push(" GROUP BY d.id;");

    let results = query
        .build_query_as::<Deal>()
        .fetch_all(&database)
        .await
        .map_err(AllDealsError::DatabaseFailure)?;
    let mut deals: Vec<ApiAllDealsResponse> =
        results.into_iter().map(ApiAllDealsResponse::from).collect();

    if deals.is_empty() {
        return Ok((StatusCode::OK, Json(Vec::<ApiAllDealsResponse>::new())).into_response());
    }

    // want to get the oldest snapshot for the deal
    let mut query = sqlx::QueryBuilder::new(
        "
        SELECT * FROM snapshots AS s
            JOIN snapshot_segment_associations AS ssa ON ssa.snapshot_id = s.id
            JOIN snapshot_segments AS ss  ON ssa.segment_id = ss.id
            WHERE deal_id IN (",
    );

    let mut separated_values = query.separated(", ");
    for deal in deals.iter() {
        separated_values.push_bind(&deal.id);
    }
    query.push(") ORDER BY s.created_at ASC LIMIT 1;");

    let snapshot = query
        .build_query_as::<SnapshotCreation>()
        .fetch_one(&database)
        .await
        .map_err(AllDealsError::DatabaseFailure)?;
    for deal in deals.iter_mut() {
        deal.requested_at = Some(snapshot.created_at);
        // the time by which the deal should be accepted,
        // not to be confused with the accepted_by field in the deals table
        // which shows the storage provider that accepted the deals
        deal.accept_by = ApiAllDealsResponse::accept_by(&snapshot.created_at);
        deal.seal_by = ApiAllDealsResponse::seal_by(deal);
    }
    Ok((StatusCode::OK, Json(deals)).into_response())
}

#[derive(Debug, thiserror::Error)]
pub enum AllDealsError {
    #[error("failed to query the database: {0}")]
    DatabaseFailure(sqlx::Error),
    #[error("query error: {0}")]
    QueryError(DealStateError),
    #[error("could not parse timestamp: {0}")]
    ComponentRange(#[from] ComponentRange),
    #[error("could not parse timestamp: {0}")]
    ParseIntError(#[from] ParseIntError),
}

impl IntoResponse for AllDealsError {
    fn into_response(self) -> Response {
        match self {
            AllDealsError::QueryError(err) => {
                tracing::error!("Bad request on looking up all deals: {err}");
                let err_msg = serde_json::json!({"msg": "Invalid deal state"});
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

#[derive(Debug, sqlx::FromRow)]
pub struct SnapshotCreation {
    pub created_at: OffsetDateTime,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiAllDealsResponse {
    pub id: String,
    pub state: DealState,
    pub size: i64,
    pub payment: i64,

    #[serde(
        skip_serializing_if = "Option::is_none",
        with = "time::serde::rfc3339::option",
        default
    )]
    pub accept_by: Option<OffsetDateTime>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        with = "time::serde::rfc3339::option",
        default
    )]
    pub requested_at: Option<OffsetDateTime>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        with = "time::serde::rfc3339::option",
        default
    )]
    pub accepted_at: Option<OffsetDateTime>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        with = "time::serde::rfc3339::option",
        default
    )]
    pub seal_by: Option<OffsetDateTime>,
}

impl From<Deal> for ApiAllDealsResponse {
    fn from(value: Deal) -> Self {
        let payment = ApiAllDealsResponse::calculate_payment_amount(&value);
        Self {
            id: value.id,
            state: value.state,
            size: value.size,
            payment,
            accepted_at: value.accepted_at,
            accept_by: None,
            seal_by: None,
            requested_at: None,
        }
    }
}

impl ApiAllDealsResponse {
    fn calculate_payment_amount(value: &Deal) -> i64 {
        ((value.size as f64 / TB_IN_BYTES) * PRICE_USD_PER_TB * 100.0).round() as i64
    }
    fn seal_by(deal: &ApiAllDealsResponse) -> Option<OffsetDateTime> {
        if let Some(accepted_at) = deal.accepted_at {
            return Some(accepted_at.add(Duration::days(DEAL_SEAL_MAX_DELAY_DAYS)));
        }
        None
    }
    fn accept_by(deal_request_date: &OffsetDateTime) -> Option<OffsetDateTime> {
        Some(deal_request_date.add(Duration::days(DEAL_SEAL_MAX_DELAY_DAYS)))
    }
}

#[cfg(test)]
mod tests {
    use axum::extract::Query;

    use crate::api::deals::all_deals::{handler, ApiAllDealsResponse, DealQuery};
    use crate::api::models::ApiDeal;
    use crate::app::mock_app_state;
    use crate::database::models::{Deal, DealState};
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
            Query(DealQuery { state: None }),
        )
        .await;

        let deals: Vec<ApiDeal> = deserialize_result(res).await;
        assert_eq!(deals.len(), 2);
        assert!(deals.iter().all(|deal| deal.state == DealState::Active));
        // because of the number of blocks in create_deal() not because there are two deals
        assert!(deals.iter().all(|deal| deal.size == 2 * BLOCK_SIZE));
        assert!(deals.iter().all(|deal| created_deals.contains(&deal.id)));
    }
    #[tokio::test]
    async fn test_calculate_payment_amount() {
        let deal = Deal {
            id: "test".to_string(),
            state: DealState::Active,
            size: 1_099_511_627_776, // 1 TB
            accepted_by: None,
            accepted_at: None,
        };
        let payment = ApiAllDealsResponse::calculate_payment_amount(&deal);
        assert_eq!(payment, 250); // $2.5 per TB

        let deal = Deal {
            id: "test".to_string(),
            state: DealState::Active,
            size: 2_199_023_255_552, // 2 TB
            accepted_by: None,
            accepted_at: None,
        };
        let payment = ApiAllDealsResponse::calculate_payment_amount(&deal);
        assert_eq!(payment, 500); // $2.5 per TB

        let deal = Deal {
            id: "test".to_string(),
            state: DealState::Active,
            size: 549_755_813_888, // 0.5 TB
            accepted_by: None,
            accepted_at: None,
        };
        let payment = ApiAllDealsResponse::calculate_payment_amount(&deal);
        assert_eq!(payment, 125); // $2.5 per TB

        let deal_half_cent = Deal {
            id: "test_half_cent".to_string(),
            state: DealState::Active,
            size: 1_789_569_706, // 0.00166665 TB
            accepted_by: None,
            accepted_at: None,
        };
        let payment_half_cent = ApiAllDealsResponse::calculate_payment_amount(&deal_half_cent);
        assert_eq!(payment_half_cent, 0); // $0.005 rounded down to $0
    }

    #[tokio::test]
    async fn test_retrieve_active_deals() {
        let db = test_helpers::setup_database().await;
        let mut conn = db.acquire().await.expect("connection");

        let created_deals = setup_deals(&mut conn).await.unwrap();

        let res = handler(
            StorageProviderIdentity::default(),
            mock_app_state(db.clone()),
            Query(DealQuery {
                state: Some("active".to_string()),
            }),
        )
        .await;

        let deals: Vec<ApiAllDealsResponse> = deserialize_result(res).await;
        assert!(deals.iter().all(|deal| deal.state == DealState::Active));
        assert!(deals.iter().all(|deal| created_deals.contains(&deal.id)));
    }

    #[tokio::test]
    async fn test_retrieve_non_active_deals() {
        let db = test_helpers::setup_database().await;
        let mut conn = db.acquire().await.expect("connection");

        let created_deals = setup_deals(&mut conn).await.unwrap();

        let res = handler(
            StorageProviderIdentity::default(),
            mock_app_state(db.clone()),
            Query(DealQuery {
                state: Some("accepted".to_string()),
            }),
        )
        .await;

        let deals: Vec<ApiAllDealsResponse> = deserialize_result(res).await;
        assert!(deals.iter().all(|deal| deal.state == DealState::Accepted));
        assert!(deals.iter().all(|deal| created_deals.contains(&deal.id)));
    }
}
