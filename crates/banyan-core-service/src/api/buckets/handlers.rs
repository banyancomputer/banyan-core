use axum::extract::{self, BodyStream, Path};
use axum::headers::{ETag, IfNoneMatch};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::TypedHeader;
use futures_util::TryStreamExt;
use object_store::ObjectStore;
use tokio::io::{AsyncWrite, AsyncWriteExt};
use uuid::Uuid;
use validify::Validate;

use crate::api::buckets::models::*;
use crate::api::buckets::requests::*;
use crate::api::buckets::responses::*;
use crate::extractors::{ApiToken, DataStore, DbConn};

pub async fn create(
    api_token: ApiToken,
    mut db_conn: DbConn, 
    extract::Json(new_bucket): extract::Json<CreateBucket>,
) -> Response {
    if let Err(errors) = new_bucket.validate() {
        return (
            StatusCode::BAD_REQUEST,
            format!("errors: {:?}", errors.errors()),
        )
            .into_response();
    }

    let maybe_bucket = sqlx::query_as!(
        CreatedResource,
        r#"INSERT INTO buckets (account_id, friendly_name, type) VALUES ($1, $2, $3) RETURNING id;"#,
        api_token.subject,
        new_bucket.friendly_name,
        new_bucket.r#type,
    )
    .fetch_one(&mut *db_conn.0)
    .await;

    let created_bucket = match maybe_bucket {
        Ok(cb) => cb,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "unable to create new bucket",
            )
                .into_response();
        }
    };

    let response = MinimalBucket {
        id: created_bucket.id,

        friendly_name: new_bucket.friendly_name,
        r#type: new_bucket.r#type,

        meta_data_cid: None,
    };

    (StatusCode::OK, axum::Json(response)).into_response()
}

pub async fn destroy(
    //_api_token: ApiToken,
    Path(_bucket_id): Path<Uuid>,
    //_if_match: Option<TypedHeader<IfMatch>>,
) -> Response {
    (StatusCode::OK, "todo").into_response()
}

pub async fn index(_api_token: ApiToken) -> Response {
    let bucket_list = vec![
        MinimalBucket {
            id: "79bfee96-0a93-4f79-87d1-212675823d6a".to_string(),

            friendly_name: "test interactive bucket".to_string(),
            r#type: BucketType::Interactive,

            meta_data_cid: Some(
                "bafybeihdwdcefgh4dqkjv67uzcmw7ojee6xedzdetojuzjevtenxquvyku".to_string(),
            ),
        },
        MinimalBucket {
            id: "7bce1c56-71b9-4147-80d4-7519a7e98bd3".to_string(),
            friendly_name: "test backup bucket".to_string(),
            r#type: BucketType::Backup,
            meta_data_cid: Some("QmdfTbBqBPQ7VNxZEYEj14VmRuZBkqFbiwReogJgS1zR1n".to_string()),
        },
    ];

    (StatusCode::OK, axum::Json(bucket_list)).into_response()
}

pub async fn publish_metadata(
    //_api_token: ApiToken,
    Path(bucket_id): Path<Uuid>,
    //_if_match: Option<TypedHeader<IfMatch>>,
    store: DataStore,
    stream: BodyStream,
) -> Response {
    // todo: authorization
    // todo: If-Match matches existing version abort

    let file_name = format!("{bucket_id}/{}.car", Uuid::new_v4());
    let file_path = object_store::path::Path::from(file_name.as_str());

    let (upload_id, mut writer) = match store.put_multipart(&file_path).await {
        Ok(mp) => mp,
        Err(_err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "unable to store uploaded file",
            )
                .into_response();
        }
    };

    let file_hash = match handle_upload(&mut writer, stream).await {
        Ok(fh) => {
            writer
                .shutdown()
                .await
                .expect("upload finalization to succeed");
            fh
        }
        Err(_err) => {
            store
                .abort_multipart(&file_path, &upload_id)
                .await
                .expect("aborting to success");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "unable to process upload",
            )
                .into_response();
        }
    };

    (StatusCode::OK, file_hash).into_response()
}

async fn handle_upload(
    writer: &mut Box<dyn AsyncWrite + Unpin + Send>,
    mut stream: BodyStream,
) -> Result<String, ()> {
    let mut hasher = blake3::Hasher::new();

    while let Some(chunk) = stream.try_next().await.transpose() {
        let chunk = chunk.expect("an available chunk (todo remove this)");

        hasher.update(&chunk);
        writer
            .write_all(&chunk)
            .await
            .expect("the write to succeed (todo remove this)");
    }

    let hash = hasher.finalize();

    Ok(hash.to_string())
}

pub async fn show(
    //_api_token: ApiToken,
    Path(bucket_id): Path<Uuid>,
    if_none_match: Option<TypedHeader<IfNoneMatch>>,
) -> Response {
    if let Some(TypedHeader(etag_hdr)) = if_none_match {
        let current_etag: ETag = "\"bafybeihdwdcefgh4dqkjv67uzcmw7ojee6xedzdetojuzjevtenxquvyku\""
            .parse()
            .expect("valid etag");

        if etag_hdr.precondition_passes(&current_etag) {
            tracing::info!("would return not modified");
            return (StatusCode::NOT_MODIFIED, "hasn't changed").into_response();
        }
    }

    let bucket = DetailedBucket {
        id: bucket_id.to_string(),
        friendly_name: "test interactive bucket".to_string(),
        r#type: BucketType::Interactive,

        meta_data_cid: Some(
            "bafybeihdwdcefgh4dqkjv67uzcmw7ojee6xedzdetojuzjevtenxquvyku".to_string(),
        ),
        public_keys: vec![
            PublicKeySummary {
                fingerprint: "0b:9e:89:30:d9:3d:36:17:f6:ca:43:ad:bf:b7:8f:32:97:40:39:f2"
                    .to_string(),
                status: PublicKeyStatus::Approved(ProtectedKey(
                    "YSBzZWNyZXQga2V5IGVuY3J5cHRlZCB3aXRoIGEgcHVibGljIGtleQo=".to_string(),
                )),
            },
            PublicKeySummary {
                fingerprint: "a3:b5:9e:5f:e8:84:ee:1f:34:d9:8e:ef:85:8e:3f:b6:62:ac:10:4a"
                    .to_string(),
                status: PublicKeyStatus::Pending,
            },
        ],
    };

    (StatusCode::OK, axum::Json(bucket)).into_response()
}
