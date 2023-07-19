use axum::extract::{self, Json};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use jsonwebtoken::{encode, get_current_timestamp, Header};
use serde::Serialize;
use sqlx::FromRow;

use crate::api::auth::requests::RegisterDeviceKey;
use crate::api::auth::AuthError;
use crate::api::ErrorResponse;
use crate::extractors::{ApiToken, DbConn, SigningKey};

const FAKE_REGISTRATION_MAXIMUM_DURATION: u64 = 60 * 60 * 24 * 28; // four weeks, should be good
                                                                   // enough for any development
#[derive(FromRow)]
struct CreatedAccount {
    id: String,
}

#[derive(Serialize)]
struct NewAccount {
    id: String,
    token: String,
}

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

    let api_token = ApiToken {
        nonce: Some(generate_nonce()),

        expiration: get_current_timestamp() + FAKE_REGISTRATION_MAXIMUM_DURATION,
        not_before: get_current_timestamp(),

        audience: "banyan-platform".into(),
        subject: created_account.id.clone(),
    };

    if let Err(err) = encode(&Header::default(), &api_token, &signing_key.0) {
        tracing::error!("error: {err}");
    }

    //match encode(&Header::default(), &api_token, &signing_key.0) {
    //    Ok(token_contents) => Json(NewAccount {
    //        id: uuid,
    //        token: token_contents,
    //    })
    //    .into_response(),
    //    Err(_) => ErrorResponse::from(AuthError).into_response(),
    //}

    Json(NewAccount { id: created_account.id, token: "test token".to_string() }).into_response()
}

pub async fn register_device_key(
    extract::Json(new_device_key): extract::Json<RegisterDeviceKey>,
) -> Response {
    let _public_key_to_register = new_device_key.public_key();

    (StatusCode::NOT_IMPLEMENTED, "todo").into_response()
}

fn generate_nonce() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    (0..8)
        .map(|_| rng.gen_range(0..256))
        .map(|b| format!("{:02x}", b))
        .collect()
}
