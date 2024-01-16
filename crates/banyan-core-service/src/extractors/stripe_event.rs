use async_trait::async_trait;
use axum::Json;
use axum::extract::{FromRef, FromRequest};
use axum::response::{IntoResponse, Response};
use http::{Request, StatusCode};
use stripe::Webhook;

use crate::app::Secrets;

const STRIPE_SIGNATURE_HDR: &str = "stripe-signature";

/// A wrapper around a validated event webhook coming from Stripe. 
pub struct StripeEvent(pub stripe::Event);

#[async_trait]
impl<S, B> FromRequest<S, B> for StripeEvent
where
    Secrets: FromRef<S>,
    String: FromRequest<S, B>,
    B: Send + 'static,
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request(req: Request<B>, state: &S) -> Result<Self, Self::Rejection> {
        let signature_hdr_val = match req.headers().get(STRIPE_SIGNATURE_HDR) {
            Some(sig) => sig.to_owned(),
            None => {
                let err_msg = serde_json::json!({"msg": "no signature provided"});
                return Err((StatusCode::BAD_REQUEST, Json(err_msg)).into_response());
            },
        };

        let signature = match signature_hdr_val.to_str() {
            Ok(sig) => sig,
            Err(_) => {
                let err_msg = serde_json::json!({"msg": "invalid characters in signature string"});
                return Err((StatusCode::BAD_REQUEST, Json(err_msg)).into_response());
            },
        };

        // The body shouldn't be parsed until after the signature has been verified
        let request = match String::from_request(req, state).await {
            Ok(req) => req,
            Err(_) => {
                let err_msg = serde_json::json!({"msg": "unable to retrieve client request body"});
                return Err((StatusCode::BAD_REQUEST, Json(err_msg)).into_response());
            },
        };

        let secrets = Secrets::from_ref(state);
        let stripe_key = match secrets.stripe_secret() {
            Some(key) => key,
            None => {
                let err_msg = serde_json::json!({"msg": "server is not setup to process stripe events"});
                return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response());
            },
        };

        let event = match Webhook::construct_event(&request, &signature, stripe_key.key()) {
            Ok(e) => e,
            Err(err) => {
                tracing::error!("failed to construct stripe webhook: {err}");
                let err_msg = serde_json::json!({"msg": "webhook event was poorly formed or had a bad signature."});
                return Err((StatusCode::BAD_REQUEST, Json(err_msg)).into_response());
            }
        };

        Ok(StripeEvent(event))
    }
}
