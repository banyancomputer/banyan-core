use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use uuid::Uuid;

use crate::api::models::ApiInvoice;
use crate::app::AppState;
use crate::database::models::{Invoice, InvoiceStatus, PriceUnits};
use crate::extractors::UserIdentity;

pub async fn handler(
    user_id: UserIdentity,
    State(state): State<AppState>,
    Path(invoice_id): Path<Uuid>,
) -> Result<Response, SingleInvoiceError> {
    let database = state.database();
    let mut conn = database.acquire().await?;

    let user_id = user_id.id().to_string();

    let invoice_id_str = invoice_id.to_string();
    let invoice = sqlx::query_as!(
        Invoice,
        r#"SELECT id, amount_due as 'amount_due: PriceUnits', status as 'status: InvoiceStatus', created_at
             FROM invoices
             WHERE id = $1 AND user_id = $2;"#,
        invoice_id_str,
        user_id,
    )
    .fetch_optional(&mut *conn)
    .await?
    .ok_or(SingleInvoiceError::NotFound)?;

    let api_invoice = ApiInvoice::from(invoice);

    Ok((StatusCode::OK, Json(api_invoice)).into_response())
}

#[derive(Debug, thiserror::Error)]
pub enum SingleInvoiceError {
    #[error("database query failed: {0}")]
    DatabaseFailure(#[from] sqlx::Error),

    #[error("subscription not found")]
    NotFound,
}

impl IntoResponse for SingleInvoiceError {
    fn into_response(self) -> Response {
        match self {
            SingleInvoiceError::DatabaseFailure(_) => {
                tracing::error!("error from database: {self}");
                let err_msg = serde_json::json!({"msg": "backend service experienced an issue servicing the request"});
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
            }
            SingleInvoiceError::NotFound => {
                let err_msg = serde_json::json!({"msg": "not found"});
                (StatusCode::NOT_FOUND, Json(err_msg)).into_response()
            }
        }
    }
}
