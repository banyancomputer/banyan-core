use async_trait::async_trait;
use axum::extract::{FromRef, FromRequest};
use axum::response::Response;
use http::Request;

use crate::app::Secrets;
use crate::database::Database;

/// A wrapper around a validated event webhook coming from Stripe. 
pub struct StripeEvent(stripe::Event);

impl StripeEvent {
    pub fn event(&self) -> &stripe::Event {
        &self.0
    }
}

#[async_trait]
impl<S, B> FromRequest<S, B> for StripeEvent
where
    Database: FromRef<S>,
    Secrets: FromRef<S>,
    String: FromRequest<S, B>,
    B: Send + 'static,
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request(_req: Request<B>, _state: &S) -> Result<Self, Self::Rejection> {
        todo!()
    }
}
