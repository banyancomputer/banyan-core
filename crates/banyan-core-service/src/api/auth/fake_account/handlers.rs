use axum::extract::{self, Json};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use openssl::bn::BigNumContext;
use openssl::ec::PointConversionForm;
use openssl::pkey::PKey;

use crate::db::models;
use crate::extractors::DbConn;

use crate::api::auth::fake_account::{requests, responses};

/// Create a fake account for testing pruposes -- bypasses oauth
pub async fn create(
    mut db_conn: DbConn,
    extract::Json(create_fake_account): extract::Json<requests::CreateFakeAccount>,
) -> Response {
    // Create a new user
    let maybe_user = sqlx::query_as!(
        models::CreatedResource,
        r#"INSERT INTO users DEFAULT VALUES RETURNING id;"#,
    )
    .fetch_one(&mut *db_conn.0)
    .await;

    let created_user = match maybe_user {
        Ok(cu) => cu,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "unable to create new user",
            )
                .into_response();
        }
    };

    // Create a new account
    let maybe_account = sqlx::query_as!(
        models::CreatedResource,
        r#"INSERT INTO accounts (userId, type, provider, providerAccountId) VALUES (?, "oauth", "not-google", 100033331337) RETURNING id;"#,
        created_user.id,
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

    // Fingerprint the provided device api key
    let device_api_key_pem = create_fake_account.device_api_key_pem();
    let parsed_public_key =
        PKey::public_key_from_pem(device_api_key_pem.as_ref()).expect("parsing public key");
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

    // Create a new device api key
    let maybe_device_api_key = sqlx::query_as!(
        models::CreatedResource,
        r#"INSERT INTO device_api_keys (account_id, fingerprint, pem) VALUES ($1, $2, $3) RETURNING id;"#,
        created_account.id,
        fingerprint,
        device_api_key_pem,
    )
    .fetch_one(&mut *db_conn.0)
    .await;

    let _ = match maybe_device_api_key {
        Ok(cda) => cda,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "unable to create new device api key",
            )
                .into_response();
        }
    };

    Json(responses::CreateFakeAccount {
        id: created_account.id,
    }).into_response()
}
