use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt;

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use cid::Cid;
use serde::{Deserialize, Deserializer};

use crate::extractors::{ApiToken, DbConn};

const NA_LABEL: &str = "NA";
pub type LocationRequest = Vec<SerializedCid>;

#[derive(serde::Serialize)]
struct InvalidCid {
    msg: String,
}

pub async fn handler(
    api_token: ApiToken,
    mut db_conn: DbConn,
    Json(request): Json<LocationRequest>,
) -> Response {
    let account_id = api_token.subject();
    let mut result_map = HashMap::new();
    for serialized_cid in &request {
        let normalized_cid = Cid::from(serialized_cid)
            .to_string_of_base(cid::multibase::Base::Base64Url)
            .expect("cid should be valid");
        let block_locations = match sqlx::query!(
            r#"SELECT sh.url
            FROM block_locations bl
            JOIN metadata m ON bl.metadata_id = m.id
            JOIN buckets b ON m.bucket_id = b.id
            JOIN storage_hosts sh ON bl.storage_host_id = sh.id
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
                .push(serialized_cid.to_string());
        } else {
            for location in block_locations {
                result_map
                    .entry(location.url)
                    .or_insert_with(Vec::new)
                    .push(serialized_cid.to_string());
            }
        }
    }
    (StatusCode::OK, Json(result_map)).into_response()
}

pub struct SerializedCid(Cid);

impl fmt::Display for SerializedCid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl SerializedCid {
    pub fn from_str(s: &str) -> Result<Self, cid::Error> {
        let cid = Cid::try_from(s)?;
        Ok(Self(cid))
    }
}

impl From<&SerializedCid> for Cid {
    fn from(serialized_cid: &SerializedCid) -> Self {
        serialized_cid.0
    }
}

impl<'de> Deserialize<'de> for SerializedCid {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(SerializedCid(
            Cid::try_from(s).map_err(serde::de::Error::custom)?,
        ))
    }
}
