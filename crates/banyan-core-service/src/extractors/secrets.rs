use std::convert::Infallible;

use axum::async_trait;
use axum::extract::{FromRef, FromRequestParts};
use http::request::Parts;

use crate::app::{Secrets, AppState};

#[async_trait]
impl FromRequestParts<AppState> for Secrets {
    type Rejection = Infallible;

    async fn from_request_parts(
        _parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        Ok(Secrets::from_ref(state))
    }
}
