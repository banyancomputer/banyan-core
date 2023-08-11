use axum::extract::{self, Json};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use openssl::bn::BigNumContext;
use openssl::ec::PointConversionForm;
use openssl::pkey::PKey;

use crate::db::models;
use crate::extractors::{ApiToken, DbConn};

use crate::api::auth::device_api_key::{requests, responses};

/// Register a new device api key with an account
pub async fn create(
    api_token: ApiToken,
    mut db_conn: DbConn,
    extract::Json(create_device_api_key): extract::Json<requests::CreateDeviceApiKey>,
) -> Response {
    let account_id = api_token.subject;
    let pem = create_device_api_key.pem();
    let parsed_public_key =
        PKey::public_key_from_pem(pem.as_ref()).expect("parsing public key");
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
        account_id,
        fingerprint,
    })
    .into_response()
}
