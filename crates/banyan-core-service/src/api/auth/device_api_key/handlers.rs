use axum::extract::{self, Json, Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;

use crate::app_state::{AppState, RegistrationEvent};
use crate::db::models;
use crate::extractors::{ApiToken, DbConn};
use openssl::bn::BigNumContext;
use openssl::ec::PointConversionForm;
use openssl::pkey::PKey;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use crate::api::auth::device_api_key::{requests, responses};

/// Register a new device api key with an account
pub async fn create(
    api_token: ApiToken,
    mut db_conn: DbConn,
    extract::Json(create_device_api_key): extract::Json<requests::CreateDeviceApiKey>,
) -> impl IntoResponse {
    let account_id = api_token.subject;
    let pem = create_device_api_key.pem();

    let parsed_public_key = PKey::public_key_from_pem(pem.as_ref()).expect("parsing public key");
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
        models::CreatedResource,
        r#"INSERT INTO device_api_keys (account_id, fingerprint, pem) VALUES ($1, $2, $3) RETURNING id;"#,
        account_id,
        fingerprint,
        pem
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

    Json(responses::CreateDeviceApiKey {
        id: created_device_key.id,
        fingerprint,
    })
    .into_response()
}

pub async fn read(
    api_token: ApiToken,
    mut db_conn: DbConn,
    Path(device_api_key_id): Path<Uuid>,
) -> impl IntoResponse {
    let account_id = api_token.subject;
    let id = device_api_key_id.to_string();

    let maybe_device_key = sqlx::query_as!(
        models::DeviceApiKey,
        r#"SELECT id, account_id, fingerprint, pem FROM device_api_keys WHERE id = $1 AND account_id = $2;"#,
        id,
        account_id
    )
    .fetch_one(&mut *db_conn.0)
    .await;

    let device_key = match maybe_device_key {
        Ok(dk) => dk,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("unable to read device key: {err}"),
            )
                .into_response();
        }
    };

    Json(responses::ReadDeviceApiKey {
        id: device_key.id,
        fingerprint: device_key.fingerprint,
        pem: device_key.pem,
    })
    .into_response()
}

// TODO: pagination
pub async fn read_all(api_token: ApiToken, mut db_conn: DbConn) -> impl IntoResponse {
    let account_id = api_token.subject;
    let maybe_device_keys = sqlx::query_as!(
        models::DeviceApiKey,
        r#"SELECT id, account_id, fingerprint, pem FROM device_api_keys WHERE account_id = $1;"#,
        account_id
    )
    .fetch_all(&mut *db_conn.0)
    .await;

    let device_keys = match maybe_device_keys {
        Ok(dks) => dks,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("unable to read device keys: {err}"),
            )
                .into_response();
        }
    };

    Json(responses::ReadDeviceApiKeys(
        device_keys
            .into_iter()
            .map(|dk| responses::ReadDeviceApiKey {
                id: dk.id,
                fingerprint: dk.fingerprint,
                pem: dk.pem,
            })
            .collect(),
    ))
    .into_response()
}

pub async fn delete(
    api_token: ApiToken,
    mut db_conn: DbConn,
    Path(device_api_key_id): Path<Uuid>,
) -> impl IntoResponse {
    let account_id = api_token.subject;
    let id = device_api_key_id.to_string();

    let maybe_device_key = sqlx::query_as!(
        models::DeviceApiKey,
        r#"DELETE FROM device_api_keys WHERE id = $1 AND account_id = $2 RETURNING id, account_id, fingerprint, pem;"#,
        id,
        account_id
    )
    .fetch_one(&mut *db_conn.0)
    .await;

    let device_key = match maybe_device_key {
        Ok(dk) => dk,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("unable to delete device key: {err}"),
            )
                .into_response();
        }
    };

    Json(responses::DeleteDeviceApiKey {
        id: device_key.id,
        fingerprint: device_key.fingerprint,
    })
    .into_response()
}

pub async fn end_regwait(
    State(mut state): State<AppState>,
    Path(fingerprint): Path<String>,
) -> impl IntoResponse {
    tracing::info!("hit reg complete for '{fingerprint}'");

    let chan_lock = match state.registration_channels.remove(&fingerprint) {
        Some(channel) => channel,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"msg": "not found 1"})),
            )
                .into_response()
        }
    };

    let channel = match chan_lock.lock() {
        Ok(mut chan) => chan.take().unwrap(),
        _ => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"msg": "not found 2"})),
            )
                .into_response()
        }
    };

    match channel.send(RegistrationEvent::Rejected) {
        Ok(_) => (StatusCode::NO_CONTENT, ()).into_response(),
        Err(_) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"msg": "not found 3"})),
        )
            .into_response(),
    }
}

pub async fn start_regwait(
    State(mut state): State<AppState>,
    Path(fingerprint): Path<String>,
) -> impl IntoResponse {
    let (sender, receiver) = tokio::sync::oneshot::channel();
    state
        .registration_channels
        .insert(fingerprint, Arc::new(Mutex::new(Some(sender))));

    match tokio::time::timeout(tokio::time::Duration::from_secs(30), receiver).await {
        Ok(chan_result) => match chan_result {
            Ok(RegistrationEvent::Approved(uuid)) => (
                StatusCode::OK,
                Json(serde_json::json!({"account_id": uuid})),
            )
                .into_response(),
            _ => (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({"msg": "device registration rejected"})),
            )
                .into_response(),
        },
        Err(_) => (
            StatusCode::REQUEST_TIMEOUT,
            Json(serde_json::json!({"msg": "device registration took too long"})),
        )
            .into_response(),
    }
}
