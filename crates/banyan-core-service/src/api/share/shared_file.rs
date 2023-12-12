use std::borrow::Cow;
use std::thread;

use async_trait::async_trait;
use axum::extract::{Json, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use bytes::BytesMut;
use cid::multibase::Base;
use futures::stream::{self};
use jwt_simple::prelude::*;
use reqwest::Client;
use tokio::sync::mpsc;
use tokio::time::{timeout, Duration as TokioDuration};
use url::Url;

// TODO: rid ourselves of anyhow
use anyhow::Result as AnyResult;
use banyan_cli::prelude::filesystem::sharing::{SharedFile, SharingError};
use wnfs::{
    common::BlockStore,
    libipld::{cid::Cid, error::SerdeError, serde as ipld_serde, IpldCodec},
    private::{share::SharePayload, PrivateForest, PrivateNode},
};

use crate::app::{AppState, ServiceKey};
use crate::database::Database;

const CHANNEL_TIMEOUT_SECS: u64 = 30;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct SharedFileQuery {
    pub payload: String,
}

#[axum::debug_handler]
pub async fn handler(
    State(state): State<AppState>,
    Query(payload): Query<SharedFileQuery>,
) -> Result<Response, SharedFileError> {
    let (tx, mut rx) = mpsc::channel(1);
    let database = state.database();
    let service_name = state.service_name().to_string();
    let service_key = state.secrets().service_key();
    let shared_file = SharedFile::import_b64_url(payload.payload)?;

    // Wnfs relies on Rc, so we need to spawn this fetching task on a separate thread
    thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async move {
            let result = fetch_data(database, service_name, service_key, shared_file).await;
            if let Err(e) = tx.send(result).await {
                tracing::error!(
                    "share call failed to send result back to main thread: {}",
                    e
                );
            }
        })
    });

    // Await the response with a timeout from the channel
    let response = timeout(TokioDuration::from_secs(CHANNEL_TIMEOUT_SECS), rx.recv()).await;

    // Match on the response and return the file data as a stream if successful
    match response {
        Ok(Some(Ok(data))) => {
            // TODO: Just did this to compile, does anyone have thoughts on how to do this better?
            let data_stream = stream::iter(
                data.into_iter()
                    .map(|item| Ok::<_, std::io::Error>(BytesMut::from(&[item][..]).freeze())),
            );

            let response = axum::body::StreamBody::new(data_stream);
            Ok((StatusCode::OK, response).into_response())
        }
        Ok(Some(Err(e))) => Err(e),
        Ok(None) => Err(SharedFileError::ChannelClosed),
        Err(_) => Err(SharedFileError::Timeout),
    }
}

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum SharedFileError {
    #[error("failed database operation: {0}")]
    Database(#[from] sqlx::Error),
    #[error("cid error: {0}")]
    Cid(#[from] cid::Error),
    #[error("failed to send request to storage host: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("jwt error: {0}")]
    Jwt(#[from] jwt_simple::JWTError),
    #[error("url error: {0}")]
    Url(#[from] url::ParseError),
    #[error("http error: {0} response from {1}")]
    Http(http::StatusCode, Url),
    #[error("could not decode base64url share payload: {0}")]
    UnableToDecodePayload(#[from] SharingError),
    #[error("could not load forest: {0}")]
    UnableToLoadForest(#[from] SerdeError),
    #[error("encountered temporal share")]
    TemporalShare,
    // Every wnfs error is an anyhow error, so we can just wrap it
    #[error("wnfs error: {0}")]
    Wnfs(#[from] anyhow::Error),
    #[error("channel closed unexpectedly")]
    ChannelClosed,
    #[error("operation timed out")]
    Timeout,
}

impl IntoResponse for SharedFileError {
    fn into_response(self) -> Response {
        use SharedFileError as SFE;

        match &self {
            SFE::Database(_)
            | SFE::Cid(_)
            | SFE::Reqwest(_)
            | SFE::Jwt(_)
            | SFE::Url(_)
            | SFE::Http(_, _)
            | SFE::UnableToLoadForest(_)
            | SFE::ChannelClosed
            | SFE::Timeout
            | SFE::Wnfs(_) => {
                tracing::error!("{self}");
                let err_msg = serde_json::json!({ "msg": "a backend service experienced an issue servicing the request" });
                (StatusCode::BAD_REQUEST, Json(err_msg)).into_response()
            }
            SFE::TemporalShare => {
                tracing::error!("{self}");
                let err_msg = serde_json::json!({ "msg": "temporal shares are not supported" });
                (StatusCode::BAD_REQUEST, Json(err_msg)).into_response()
            }
            SFE::UnableToDecodePayload(_) => {
                tracing::error!("{self}");
                let err_msg = serde_json::json!({ "msg": "invalid share payload" });
                (StatusCode::BAD_REQUEST, Json(err_msg)).into_response()
            }
        }
    }
}

/// Fetch the file pointed to by the shared file.
/// Queries database for block locations, then fetches the blocks from the storage hosts.
/// Authenticates with the storage hosts using the service key.
/// # Arguments
/// * `database` - The database pool
/// * `service_key` - The service key to use for auth to storage hosts
/// * `shared_file` - The shared file to fetch
/// # Returns
/// The file data as a Vec<u8>
async fn fetch_data(
    database: Database,
    service_name: String,
    service_key: ServiceKey,
    shared_file: SharedFile,
) -> Result<Vec<u8>, SharedFileError> {
    let forest_cid = shared_file.forest_cid;
    let store = ShareBlockStore::new(database, service_name, service_key)?;

    // Get the forest first
    let forest_ipld = store.get_deserializable(&forest_cid).await?;
    let forest = ipld_serde::from_ipld::<PrivateForest>(forest_ipld)?;

    // Now get the data now that we have the forest
    match shared_file.payload {
        SharePayload::Temporal(_) => Err(SharedFileError::TemporalShare),
        SharePayload::Snapshot(snapshot) => {
            let file = PrivateNode::load_from_snapshot(snapshot, &forest, &store)
                .await?
                .as_file()?;
            let data = file.get_content(&forest, &store).await?;
            Ok(data)
        }
    }
}

/// Simple wrapper around the database and service key.
/// Allows our BlockStore implementation to find and get blocks from storage hosts.
struct ShareBlockStore {
    database: Database,
    service_name: String,
    service_key: ServiceKey,
    client: Client,
}

impl ShareBlockStore {
    pub fn new(
        database: Database,
        service_name: String,
        service_key: ServiceKey,
    ) -> Result<Self, SharedFileError> {
        Ok(Self {
            database,
            service_name,
            service_key,
            client: Client::builder().build()?,
        })
    }

    /// Find which storage host has the block we're looking for
    /// # Arguments
    /// * `cid` - The CID of the block we're looking for
    pub async fn find_storage_host(&self, cid: &Cid) -> Result<StorageHostInfo, SharedFileError> {
        // Interpreting the CID as a base64url string
        let normalized_cid = cid.to_string_of_base(Base::Base64Url)?;
        let storage_host_info = sqlx::query_as!(
            StorageHostInfo,
            "SELECT sh.url, sh.name
            FROM storage_hosts sh
            JOIN block_locations bl ON sh.id = bl.storage_host_id
            JOIN blocks b ON bl.block_id = b.id
            WHERE b.cid = $1;",
            normalized_cid
        )
        .fetch_one(&self.database)
        .await
        .map_err(SharedFileError::Database)?;

        Ok(storage_host_info)
    }

    /// Get the block from the storage host
    /// # Arguments
    /// * `storage_host_info` - The storage host to get the block from
    /// * `cid` - The CID of the block to get
    /// # Returns
    /// The block data as a Vec<u8>
    pub async fn request_block(
        &self,
        storage_host_info: StorageHostInfo,
        cid: &Cid,
    ) -> Result<Vec<u8>, SharedFileError> {
        // Create claims againt the storage host
        // They only need to be valid long enough to fulfill the request, which will timeout after CHANNEL_TIMEOUT_SECS
        let mut claims = Claims::create(Duration::from_secs(CHANNEL_TIMEOUT_SECS))
            .with_audiences(HashSet::from_strings(&[storage_host_info.name]))
            .with_subject(&self.service_name)
            .invalid_before(Clock::now_since_epoch() - Duration::from_secs(30));
        claims.create_nonce();
        claims.issued_at = Some(Clock::now_since_epoch());
        // TODO: better error handling here
        let bearer_token = self.service_key.sign(claims)?;

        // Build and Attach the auth token to the request
        let url = Url::parse(&storage_host_info.url)?;
        // Tack on the api route + base32 default CID encoding
        let url = url.join(&format!("/api/v1/blocks/{cid}"))?;
        let request = self.client.get(url.clone()).bearer_auth(bearer_token);

        // Send the request and handle the response.
        // It's ok to read the entire stream here since the calling function isn't expecting one.
        let response = request.send().await?;

        if response.status().is_success() {
            let data = response.bytes().await?;
            Ok(data.to_vec())
        } else {
            Err(SharedFileError::Http(response.status(), url))
        }
    }
}

#[async_trait(?Send)]
impl BlockStore for ShareBlockStore {
    async fn get_block(&self, cid: &Cid) -> AnyResult<Cow<Vec<u8>>> {
        // Find the storage host that has the block
        let storage_host_info = self.find_storage_host(cid).await?;
        // Get the block from the storage host
        let block = self.request_block(storage_host_info, cid).await?;
        // Done!
        Ok(Cow::Owned(block))
    }

    async fn put_block(&self, _block: Vec<u8>, _codec: IpldCodec) -> AnyResult<Cid> {
        panic!("not implemented")
    }
}

#[derive(sqlx::FromRow)]
struct StorageHostInfo {
    url: String,
    name: String,
}
