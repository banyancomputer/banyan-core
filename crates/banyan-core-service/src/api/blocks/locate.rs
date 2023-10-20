use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt;

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use cid::Cid;
use serde::{Deserialize, Deserializer};

use crate::extractors::{ApiToken, DbConn};
use crate::error::CoreError;

const NA_LABEL: &str = "NA";
pub type LocationRequest = Vec<Cid>;

pub async fn handler(
    api_token: ApiToken,
    mut db_conn: DbConn,
    Json(request): Json<LocationRequest>,
) -> Response {
    let account_id = api_token.subject();
    let mut result_map = HashMap::new();
    for cid in &request {
        let normalized_cid = 
            match cid
            .to_string_of_base(cid::multibase::Base::Base64Url) {
                Ok(normalized_cid) => normalized_cid,
                Err(err) => 
                    return CoreError::generic_error(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "backend service issue",
                        Some(&format!("unable to normalize cid {err}")),
                    ).into_response()
            };
        let block_locations = match sqlx::query!(
            r#"SELECT sh.url
            FROM storage_hosts sh
            JOIN block_locations bl ON sh.id = bl.storage_host_id
            JOIN metadata m ON bl.metadata_id = m.id
            JOIN buckets b ON m.bucket_id = b.id
            JOIN blocks ON bl.block_id = blocks.id
            WHERE blocks.cid = $1
            AND b.account_id = $2
            "#,
            normalized_cid,
            account_id
        )
        .fetch_all(&mut *db_conn.0)
        .await
        {
            Ok(maybe_block_locations) => maybe_block_locations,
            Err(err) => {
                tracing::error!(
                    "unable to get block locations from block_locations table: {}",
                    err
                );
                // Push the cid onto the NA label
                vec![]
            }
        };
        if block_locations.is_empty() {
            // Push the cid onto the NA label
            result_map
                .entry(NA_LABEL.to_string())
                .or_insert_with(Vec::new)
                .push(cid);
        } else {
            for location in block_locations {
                result_map
                    .entry(location.url)
                    .or_insert_with(Vec::new)
                    .push(cid);
            }
        }
    }
    (StatusCode::OK, Json(result_map)).into_response()
}