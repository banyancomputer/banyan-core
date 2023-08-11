use axum::extract::{self, BodyStream, Path};
//use axum::headers::{ETag, IfNoneMatch};
use axum::headers::ContentType;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::TypedHeader;
use futures::{TryStream, TryStreamExt};
use object_store::ObjectStore;
use tokio::io::{AsyncWrite, AsyncWriteExt};
use uuid::Uuid;
use validify::Validate;

use crate::api::buckets::car_buffer::CarBuffer;
use crate::api::buckets::{models, requests, responses};
use crate::extractors::{ApiToken, DataStore, DbConn};

pub async fn create(
    api_token: ApiToken,
    mut db_conn: DbConn,
    extract::Json(new_bucket): extract::Json<requests::CreateBucket>,
) -> Response {
    if let Err(errors) = new_bucket.validate() {
        return (
            StatusCode::BAD_REQUEST,
            format!("errors: {:?}", errors.errors()),
        )
            .into_response();
    }

    let maybe_bucket = sqlx::query_as!(
        models::CreatedResource,
        r#"INSERT INTO buckets (account_id, name, type) VALUES ($1, $2, $3) RETURNING id;"#,
        api_token.subject,
        new_bucket.name,
        new_bucket.r#type,
    )
    .fetch_one(&mut *db_conn.0)
    .await;

    let created_bucket = match maybe_bucket {
        Ok(cb) => cb,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("unable to create new bucket: {err}"),
            )
                .into_response();
        }
    };

    if sqlx::query_as!(
        models::CreatedResource,
        r#"INSERT INTO bucket_keys (bucket_id, approved, pem) VALUES ($1, true, $2) RETURNING id;"#,
        created_bucket.id,
        new_bucket.initial_public_key,
    )
    .fetch_one(&mut *db_conn.0)
    .await
    .is_err()
    {
        // todo: should try to delete created bucket
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "unable to create public key associated with bucket",
        )
            .into_response();
    }

    let response = responses::MinimalBucket {
        id: created_bucket.id,

        name: new_bucket.name,
        r#type: new_bucket.r#type,
    };

    (StatusCode::OK, axum::Json(response)).into_response()
}

pub async fn destroy(
    _api_token: ApiToken,
    Path(_bucket_id): Path<Uuid>,
    //_if_match: Option<TypedHeader<IfMatch>>,
) -> Response {
    (StatusCode::OK, "todo").into_response()
}

pub async fn index(_api_token: ApiToken) -> Response {
    let bucket_list = vec![
        responses::MinimalBucket {
            id: "79bfee96-0a93-4f79-87d1-212675823d6a".to_string(),
            name: "test interactive bucket".to_string(),
            r#type: responses::BucketType::Interactive,
        },
        responses::MinimalBucket {
            id: "7bce1c56-71b9-4147-80d4-7519a7e98bd3".to_string(),
            name: "test backup bucket".to_string(),
            r#type: responses::BucketType::Backup,
        },
    ];

    (StatusCode::OK, axum::Json(bucket_list)).into_response()
}

// This is just the metadata car file, 128MiB for that is just a rough (what I think is generous)
// limit. We can revist this if its insufficient.
const CAR_DATA_SIZE_LIMIT: u64 = 128 * 1_024 * 1_024;

// Limit on the size of the data portion of these upload requests to protect the server from
// malicious use, 100KiB should be more than enough for now.
const REQUEST_DATA_SIZE_LIMIT: u64 = 100 * 1_024;

pub async fn publish_metadata(
    _api_token: ApiToken,

    Path(bucket_id): Path<Uuid>,
    //_if_match: Option<TypedHeader<IfMatch>>,
    store: DataStore,

    TypedHeader(content_type): TypedHeader<ContentType>,
    body: BodyStream,
) -> Response {
    // todo: authorization
    // todo: If-Match matches existing version abort

    let mime_ct = mime::Mime::from(content_type);
    let boundary = multer::parse_boundary(mime_ct).unwrap();
    let constraints = multer::Constraints::new()
        .allowed_fields(vec!["request-data", "car-upload"])
        .size_limit(
            multer::SizeLimit::new()
                .for_field("request-data", REQUEST_DATA_SIZE_LIMIT)
                .for_field("car-upload", CAR_DATA_SIZE_LIMIT),
        );

    let mut multipart = multer::Multipart::with_constraints(body, boundary, constraints);

    let request_data_field = multipart.next_field().await.unwrap().unwrap();
    // todo: validate name is request-data (request_data_field.name())
    // todo: validate type is application/json (request_data_field.content_type())
    let pbmr_bytes = request_data_field.bytes().await.unwrap();
    let _data: requests::PublishBucketMetadataRequest =
        serde_json::from_slice(&pbmr_bytes).unwrap();

    // todo: validate / store data

    // get the next field which should be our car data
    let car_stream = multipart.next_field().await.unwrap().unwrap();
    // todo: validate name is car-upload (request_data_field.name())
    // todo: validate type is "application/vnd.ipld.car; version=2" (request_data_field.content_type())

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

    let _file_hash = match handle_metadata_upload(car_stream, &mut writer).await {
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

    let response = responses::PublishBucketMetadataResponse {
        id: Uuid::new_v4().to_string(),
        state: responses::MetadataState::Pending,

        storage_host: "http://127.0.0.1:3002".to_string(),
        storage_authorization: "todo: JWT here".to_string(),
    };

    (StatusCode::OK, axum::Json(response)).into_response()
}

async fn handle_metadata_upload<S>(
    mut stream: S,
    writer: &mut Box<dyn AsyncWrite + Unpin + Send>,
) -> Result<String, ()>
where
    S: TryStream<Ok = bytes::Bytes> + Unpin,
    S::Error: std::error::Error,
{
    let mut car_buffer = CarBuffer::new();
    let mut hasher = blake3::Hasher::new();

    while let Some(chunk) = stream.try_next().await.transpose() {
        let chunk = chunk.expect("an available chunk (todo remove this)");

        hasher.update(&chunk);
        car_buffer.add_chunk(&chunk);

        //match car_buffer.parse() {
        //    Ok(Some(_md)) => {
        //        // todo: we have our metadata, do any validation we need to here
        //    }
        //    Ok(None) => (),
        //    Err(err) => {
        //        tracing::error!("received car buffer error: {err}");
        //        return Err(());
        //    }
        //}

        writer
            .write_all(&chunk)
            .await
            .expect("the write to succeed (todo remove this)");
    }

    let hash = hasher.finalize();

    Ok(hash.to_string())
}

pub async fn show(
    _api_token: ApiToken,
    Path(bucket_id): Path<Uuid>,
    //if_none_match: Option<TypedHeader<IfNoneMatch>>,
) -> Response {
    //if let Some(TypedHeader(etag_hdr)) = if_none_match {
    //    let current_etag: ETag = "\"bafybeihdwdcefgh4dqkjv67uzcmw7ojee6xedzdetojuzjevtenxquvyku\""
    //        .parse()
    //        .expect("valid etag");

    //    if etag_hdr.precondition_passes(&current_etag) {
    //        tracing::info!("would return not modified");
    //        return (StatusCode::NOT_MODIFIED, "hasn't changed").into_response();
    //    }
    //}

    let bucket = responses::DetailedBucket {
        id: bucket_id.to_string(),
        name: "test interactive bucket".to_string(),
        r#type: responses::BucketType::Interactive,

        public_keys: vec![
            responses::PublicKeySummary {
                approved: true,
                fingerprint: "<pending>".to_string(),
                pem: "<full public key>".to_string(),
            },
            responses::PublicKeySummary {
                approved: false,
                fingerprint: "<pending>".to_string(),
                pem: "<full public key>".to_string(),
            },
        ],
    };

    (StatusCode::OK, axum::Json(bucket)).into_response()
}
