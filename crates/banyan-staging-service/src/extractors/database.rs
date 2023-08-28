use axum::{async_trait, Json, RequestPartsExt};
use axum::extract::{FromRef, FromRequestParts, TypedHeader};
use axum::extract::rejection::TypedHeaderRejection;
use axum::headers::Authorization;
use axum::headers::authorization::Bearer;
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use crate::database::Database as StateDb;

#[derive(Debug)]
pub struct Database(StateDb);

#[async_trait]
impl<S> FromRequestParts<S> for Database
where
    StateDb: FromRef<S>,
    S: Send + Sync,
{
  type Rejection = ();

    async fn from_request_parts(
        _parts: &mut Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
      Ok(Database(StateDb::from_ref(state)))
    }
}
