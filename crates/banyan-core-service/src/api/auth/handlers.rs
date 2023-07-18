use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use jsonwebtoken::{encode, get_current_timestamp, EncodingKey, Header};
use uuid::Uuid;

use crate::api::auth::AuthError;
use crate::api::ErrorResponse;
use crate::extractors::{ApiToken, EXPIRATION_WINDOW_SECS, TESTING_API_KEY};

pub async fn fake_token() -> Response {
    let api_token = ApiToken {
        nonce: Some("todo-generate-random-none".to_string()),

        expiration: get_current_timestamp() + EXPIRATION_WINDOW_SECS,
        not_before: get_current_timestamp(),

        audience: "did:key:{some-kind-of-banyan-identity}".into(),
        subject: Uuid::new_v4().to_string(),
    };

    let key = EncodingKey::from_secret(TESTING_API_KEY.as_ref());

    let token_header = Header {
        kid: Some("key fingerprint for identity".to_string()),
        ..Default::default()
    };

    match encode(&token_header, &api_token, &key) {
        Ok(token_contents) => (StatusCode::OK, token_contents).into_response(),
        Err(_) => ErrorResponse::from(AuthError).into_response(),
    }
}
