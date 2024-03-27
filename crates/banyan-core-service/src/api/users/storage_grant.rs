use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use base64::{engine::general_purpose::URL_SAFE, Engine as _};
use jwt_simple::prelude::*;
use url::Url;

use crate::app::AppState;
use crate::auth::storage_ticket::StorageTicketBuilder;
use crate::extractors::ApiIdentity;

#[axum::debug_handler]
pub async fn handler(
    api_id: ApiIdentity,
    State(state): State<AppState>,
    Path(encoded_base_url): Path<String>,
) -> Response {
    let database = state.database();
    let service_key = state.secrets().service_key();

    let decoded_url_bytes = match URL_SAFE.decode(encoded_base_url) {
        Ok(bu) => bu,
        Err(_) => {
            let err_msg = serde_json::json!({"msg": "invalid base64url encoding"});
            return (StatusCode::BAD_REQUEST, Json(err_msg)).into_response();
        }
    };

    let base_url = match String::from_utf8(decoded_url_bytes) {
        Ok(url) => url,
        Err(_) => {
            let err_msg = serde_json::json!({"msg": "invalid utf-8 encoding"});
            return (StatusCode::BAD_REQUEST, Json(err_msg)).into_response();
        }
    };

    let full_domain = match Url::parse(&base_url) {
        Ok(url) => url,
        Err(_) => {
            let err_msg = serde_json::json!({"msg": "invalid url provided for host"});
            return (StatusCode::BAD_REQUEST, Json(err_msg)).into_response();
        }
    };

    if full_domain.path() != "/"
        || full_domain.query().is_some()
        || full_domain.fragment().is_some()
    {
        let err_msg = serde_json::json!({"msg": "only base hostnames are accepted"});
        return (StatusCode::BAD_REQUEST, Json(err_msg)).into_response();
    }

    let storage_host_base_url = full_domain.to_string();
    let user_id = api_id.user_id().to_string();

    let token_details = sqlx::query_as!(
        TokenDetails,
        r#"SELECT sg.id as storage_grant_id, sg.authorized_amount as authorized_amount,
                   sh.name as service_name, sh.url as service_url
               FROM storage_grants AS sg
               JOIN storage_hosts AS sh ON sh.id = sg.storage_host_id
               JOIN storage_hosts_metadatas_storage_grants AS shmsg ON shmsg.storage_grant_id = sg.id
               JOIN metadata AS m ON m.id = shmsg.metadata_id
               JOIN buckets AS b ON m.bucket_id = b.id
               WHERE b.deleted_at IS NULL
                   AND sg.redeemed_at IS NOT NULL
                   AND sg.user_id = $1
                   AND sh.url = $2
               ORDER BY sg.redeemed_at DESC
               LIMIT 1;"#,
        user_id,
        storage_host_base_url,
    )
    .fetch_optional(&database)
    .await;

    let token_details = match token_details {
        Ok(Some(token_details)) => token_details,
        Ok(None) => {
            let err_msg = serde_json::json!({"msg": "user has no grants with requested host"});
            return (StatusCode::CONFLICT, Json(err_msg)).into_response();
        }
        Err(_) => {
            let err_msg = serde_json::json!({"msg": "internal server error"});
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response();
        }
    };

    let mut ticket_builder = StorageTicketBuilder::new(api_id.ticket_subject());
    ticket_builder.add_audience(token_details.service_name);
    ticket_builder.add_authorization(
        token_details.storage_grant_id,
        token_details.service_url,
        token_details.authorized_amount,
    );
    let claim = ticket_builder.build();
    let bearer_token = match service_key.sign(claim) {
        Ok(token) => token,
        Err(_) => {
            let err_msg = serde_json::json!({"msg": "internal server error"});
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response();
        }
    };

    let resp = serde_json::json!({"token": bearer_token });
    (StatusCode::OK, Json(resp)).into_response()
}

#[derive(sqlx::FromRow)]
struct TokenDetails {
    storage_grant_id: String,

    authorized_amount: i64,

    service_name: String,
    service_url: String,
}
