use std::collections::HashMap;

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;

use crate::extractors::{ApiToken, DbConn};

#[derive(serde::Deserialize)]
pub struct LocationRequest {
    cids: Vec<String>,
}

pub async fn handler(
    api_token: ApiToken,
    mut db_conn: DbConn,
    Json(request): Json<LocationRequest>,
) -> Response {
    let account_id = api_token.subject();
    let mut result_map = HashMap::new();
    for cid in &request.cids {
        let normalized_cid = match cid::Cid::try_from(cid.to_string()) {
            Ok(cid) => {
                cid.to_string_of_base(cid::multibase::Base::Base64Url)
                    .expect("parsed cid to unparse")
            }
            Err(err) => {
                tracing::error!("unable to parse cid: {}", err);
                continue;
            }
        };

        let block_id = match sqlx::query_scalar!(
            r#"SELECT id FROM blocks WHERE cid = $1"#,
            normalized_cid
        )
        .fetch_one(&mut *db_conn.0)
        .await
        {
            Ok(block_id) => block_id,
            Err(err) => {
                tracing::error!("unable to get block id from blocks table: {}", err);
                continue;
            }
        };

        let block_location: Option<String> = match sqlx::query_scalar!(
            r#"SELECT sh.url
                FROM block_locations bl
                JOIN metadata m ON bl.metadata_id = m.id
                JOIN buckets b ON m.bucket_id = b.id
                JOIN storage_hosts sh ON bl.storage_host_name = sh.name
                WHERE bl.block_id = $1
                AND b.account_id = $2"#,
            block_id,
            account_id 
        )
        .fetch_optional(&mut *db_conn.0)
        .await
        {
            Ok(maybe_block_location) => maybe_block_location,
            Err(err) => {
                tracing::error!("unable to get block location from block_locations table: {}", err);
                continue;
            }
        };

        if let Some(location) = block_location {
            result_map.insert(cid.to_owned(), location);
        }
    }
    (StatusCode::OK, Json(result_map)).into_response()
}
