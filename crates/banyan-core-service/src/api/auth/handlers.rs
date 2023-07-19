use axum::extract;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use jsonwebtoken::{encode, get_current_timestamp, EncodingKey, Header};
use uuid::Uuid;

use crate::api::ErrorResponse;
use crate::api::auth::AuthError;
use crate::api::auth::requests::RegisterDeviceKey;
use crate::extractors::{ApiToken, EXPIRATION_WINDOW_SECS, TESTING_API_KEY};

pub async fn fake_register() -> Response {
    let api_token = ApiToken {
        nonce: Some("todo-generate-random-none".to_string()),

        expiration: get_current_timestamp() + EXPIRATION_WINDOW_SECS,
        not_before: get_current_timestamp(),

        audience: "did:key:{some-kind-of-banyan-identity}".into(),
        subject: Uuid::new_v4().to_string(),
    };

    let key = EncodingKey::from_secret(TESTING_API_KEY.as_ref());

    let token_header = Header {
        kid: Some("4e:12:43:bd:22:c6:6e:76:c2:ba:9e:dd:c1:f9:13:94:e5:7f:9f:83".to_string()),
        ..Default::default()
    };

    match encode(&token_header, &api_token, &key) {
        Ok(token_contents) => (StatusCode::OK, token_contents).into_response(),
        Err(_) => ErrorResponse::from(AuthError).into_response(),
    }
}

pub async fn register_device_key(
    extract::Json(new_device_key): extract::Json<RegisterDeviceKey>,
) -> Response {
    let _public_key_to_register = new_device_key.public_key();

    (StatusCode::NOT_IMPLEMENTED, "todo").into_response()
}
