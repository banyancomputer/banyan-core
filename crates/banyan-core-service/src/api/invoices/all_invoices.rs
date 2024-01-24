use axum::extract::{Json, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use crate::api::models::ApiInvoice;
use crate::app::AppState;
use crate::database::models::{Invoice, InvoiceStatus, PriceUnits};
use crate::extractors::UserIdentity;

pub async fn handler(
    user_id: UserIdentity,
    State(state): State<AppState>,
) -> Result<Response, AllInvoicesError> {
    let database = state.database();
    let mut conn = database.acquire().await?;

    let user_id = user_id.id().to_string();
    let invoices = sqlx::query_as!(
        Invoice,
        r#"SELECT id, amount_due as 'amount_due: PriceUnits', status as 'status: InvoiceStatus', created_at
             FROM invoices
             WHERE user_id = $1
             ORDER BY created_at DESC;"#,
        user_id,
    )
    .fetch_all(&mut *conn)
    .await?;

    let api_invoices: Vec<_> = invoices.into_iter().map(ApiInvoice::from).collect();

    Ok((StatusCode::OK, Json(api_invoices)).into_response())
}

#[derive(Debug, thiserror::Error)]
pub enum AllInvoicesError {
    #[error("database query failed: {0}")]
    DatabaseFailure(#[from] sqlx::Error),
}

impl IntoResponse for AllInvoicesError {
    fn into_response(self) -> Response {
        {
            tracing::error!("all invoices error: {self}");
            let err_msg = serde_json::json!({"msg": "backend service experienced an issue servicing the request"});
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
        }
    }
}
