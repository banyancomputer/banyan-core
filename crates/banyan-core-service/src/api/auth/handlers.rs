use axum::extract::{self, Json};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use jsonwebtoken::{encode, get_current_timestamp, Algorithm, Header};

use openssl::bn::BigNumContext;
use openssl::ec::PointConversionForm;
use openssl::pkey::PKey;

use crate::api::auth::{AuthError, models, requests, responses};
use crate::api::ErrorResponse;
use crate::extractors::{DbConn, FakeToken, SigningKey};

const FAKE_REGISTRATION_MAXIMUM_DURATION: u64 = 60 * 60 * 24 * 28; // four weeks, should be good enough between env resets

pub async fn fake_register(mut db_conn: DbConn, signing_key: SigningKey) -> Response {
    let maybe_account = sqlx::query_as!(
        models::CreatedAccount,
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
        Ok(token) => Json(responses::NewAccount {
            id: created_account.id,
            token,
        })
        .into_response(),
        Err(err) => {
            tracing::error!("unable to encode jwt: {err}");
            ErrorResponse::from(AuthError).into_response()
        }
    }
}

pub async fn register_device_key(
    api_token: FakeToken,
    mut db_conn: DbConn,
    extract::Json(new_device_key): extract::Json<requests::RegisterDeviceKey>,
) -> Response {
    let account_id = api_token.subject;
    let pem_to_register = new_device_key.public_key();

    let parsed_public_key =
        PKey::public_key_from_pem(pem_to_register.as_ref()).expect("parsing public key");
    let ec_key = parsed_public_key.ec_key().unwrap();
    let ec_group = ec_key.group();
    let mut big_num_context = BigNumContext::new().expect("big number context");

    let raw_compressed_bytes = ec_key
        .public_key()
        .to_bytes(
            ec_group,
            PointConversionForm::COMPRESSED,
            &mut big_num_context,
        )
        .expect("pub key bytes");
    let fingerprint_bytes = openssl::sha::sha1(&raw_compressed_bytes);
    let fingerprint = fingerprint_bytes
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<Vec<String>>()
        .join(":");

    let maybe_device_key = sqlx::query_as!(
        models::CreatedDeviceKey,
        r#"INSERT INTO device_api_keys (account_id, fingerprint, pem) VALUES ($1, $2, $3) RETURNING id;"#,
        account_id,
        fingerprint,
        pem_to_register
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

    Json(responses::NewDeviceKey {
        id: created_device_key.id,
        account_id,
        fingerprint,
    })
    .into_response()
}
