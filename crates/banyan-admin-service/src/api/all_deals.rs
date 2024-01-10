use axum::extract::{Query, State};
use axum::response::{IntoResponse, Response};
use axum::Json;
use http::StatusCode;

use crate::app::AppState;
use crate::client::GetAllDealsRequest;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct DealQuery {
    pub status: Option<String>,
}

pub async fn handler(
    State(state): State<AppState>,
    Query(_): Query<DealQuery>,
) -> Result<Response, ()> {
    let deals = state.client().call(GetAllDealsRequest).await.unwrap();

    Ok((StatusCode::OK, Json(deals)).into_response())
}
