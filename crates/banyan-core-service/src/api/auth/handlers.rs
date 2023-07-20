use axum::extract::{self, Json};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use jsonwebtoken::{encode, get_current_timestamp, Algorithm, Header};

use crate::api::auth::AuthError;
use crate::api::auth::models::*;
use crate::api::auth::requests::*;
use crate::api::auth::responses::*;
use crate::api::ErrorResponse;
use crate::extractors::{FakeToken, DbConn, SigningKey};

const FAKE_REGISTRATION_MAXIMUM_DURATION: u64 = 60 * 60 * 24 * 28; // four weeks, should be good enough between env resets

pub async fn fake_register(mut db_conn: DbConn, signing_key: SigningKey) -> Response {
    let maybe_account = sqlx::query_as!(
        CreatedAccount,
        r#"INSERT INTO accounts DEFAULT VALUES RETURNING id;"#
    )
    .fetch_one(&mut *db_conn.0)
    .await;

    let created_account = match maybe_account {
        Ok(ca) => ca,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "unable to create new account",
            )
                .into_response();
        }
    };

    let api_token = FakeToken {
        expiration: get_current_timestamp() + FAKE_REGISTRATION_MAXIMUM_DURATION,
        subject: created_account.id.clone(),
    };

    let header = Header {
        alg: Algorithm::ES384,
        ..Default::default()
    };

    match encode(&header, &api_token, &signing_key.0) {
        Ok(token) => Json(NewAccount { id: created_account.id, token }).into_response(),
        Err(_) => ErrorResponse::from(AuthError).into_response(),
    }
}

pub async fn register_device_key(
    api_token: FakeToken,
    mut db_conn: DbConn, 
    extract::Json(new_device_key): extract::Json<RegisterDeviceKey>,
) -> Response {
    let account_id = api_token.subject;
    let public_key_to_register = new_device_key.public_key();

    let device_key_pem = match pem::parse(public_key_to_register.as_bytes()) {
        Ok(dkp) => dkp,
        Err(err) => {
            return (StatusCode::BAD_REQUEST, err.to_string()).into_response();
        }
    };

    if device_key_pem.tag() != "PUBLIC KEY" {
        return (StatusCode::BAD_REQUEST, "not public key").into_response();
    }

    // todo: fingerprint is wrong
    let public_key_der_bytes = device_key_pem.into_contents();
    let fingerprint = public_key_der_bytes.iter().map(|byte| format!("{byte:02x}")).collect::<Vec<String>>().join(":");

    let maybe_device_key = sqlx::query_as!(
        CreatedDeviceKey,
        r#"INSERT INTO device_api_keys (account_id, fingerprint, public_key) VALUES ($1, $2, $3) RETURNING id;"#,
        account_id,
        fingerprint,
        public_key_to_register
    )
    .fetch_one(&mut *db_conn.0)
    .await;

    let created_device_key = match maybe_device_key {
        Ok(cda) => cda,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("unable to create new device key: {err}"),
            )
                .into_response();
        }
    };

    Json(NewDeviceKey {
        id: created_device_key.id,
        account_id,
        fingerprint,
    }).into_response()
}
