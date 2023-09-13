use axum::{async_trait, Json};
use axum::extract::{FromRef, FromRequestParts};
use axum::response::{IntoResponse, Response};
use sqlx::sqlite::SqlitePool;

pub struct DbConn(pub(crate) sqlx::pool::PoolConnection<sqlx::Sqlite>);

#[async_trait]
impl<S> FromRequestParts<S> for DbConn
where
    SqlitePool: FromRef<S>,
    S: Send + Sync,
{
    // TODO: better error
    type Rejection = Response;

    async fn from_request_parts(
        _parts: &mut http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let pool = SqlitePool::from_ref(state);

        let conn = pool.acquire().await.map_err(|_| {
            (
                http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"msg": "failed to acquire database connection"})),
            ).into_response()
        })?;

        Ok(Self(conn))
    }
}
