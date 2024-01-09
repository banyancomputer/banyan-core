use async_trait::async_trait;
use axum::extract::FromRequest;

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
    String: FromRequest<S, B>,
    B: Send + 'static,
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request(req: Request<B>, state: &S) -> Result<Self, Self::Rejection> {
        todo!()
    }
}
