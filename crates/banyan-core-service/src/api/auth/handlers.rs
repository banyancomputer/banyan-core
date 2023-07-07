use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use jsonwebtoken::{encode, get_current_timestamp, EncodingKey, Header};
use uuid::Uuid;

use crate::api::auth::AuthError;
use crate::api::ErrorResponse;
use crate::extractors::{ApiToken, EXPIRATION_WINDOW_SECS, TESTING_API_KEY};

pub async fn fake_token() -> Response {
    let api_token = ApiToken {
        audience: "banyan-core".into(),
        subject: Uuid::new_v4().to_string(),

        expiration: get_current_timestamp() + EXPIRATION_WINDOW_SECS,
        not_before: get_current_timestamp(),

        attenuation: vec![],
        proofs: vec![],
    };

    let key = EncodingKey::from_secret(TESTING_API_KEY.as_ref());
    let token_contents = encode(&Header::default(), &api_token, &key)
        .map_err(|_| AuthError)
        .map_err(|ae| {
            return ErrorResponse::from(ae).into_response();
        });

    (StatusCode::OK, token_contents).into_response()
}
