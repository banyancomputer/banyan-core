use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use jwt_simple::prelude::*;
use serde::Deserialize;
use url::Url;

use crate::api::models::ApiUser;
use crate::app::{AppState, ServiceKey};
use crate::auth::storage_ticket::StorageTicketBuilder;
use crate::database::models::{MetricsTraffic, User};
use crate::database::Database;
use crate::extractors::UserIdentity;

pub async fn handler(
    api_id: ApiIdentity,
    database: Database,
    service_key: ServiceKey,
    Path(base_url): Path<String>,
) -> Response {
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
               WHERE sg.user_id = $1 AND sh.url = $2"#,
        &user_id,
    )
    .fetch_one(&database)
    .await?;

    todo!()
}

#[derive(sqlx::FromRow)]
struct TokenDetails {
    storage_grant_id: String,

    authorized_amount: i64,

    service_name: String,
    service_url: String,
}
