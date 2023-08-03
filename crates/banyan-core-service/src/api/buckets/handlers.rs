use axum::extract::{self, BodyStream, Path};
//use axum::headers::{ETag, IfNoneMatch};
use axum::headers::ContentType;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::TypedHeader;
use futures_util::TryStreamExt;
use object_store::ObjectStore;
use multer::Multipart;
use tokio::io::{AsyncWrite, AsyncWriteExt};
use uuid::Uuid;
use validify::Validate;

use crate::api::buckets::car_buffer::CarBuffer;
use crate::api::buckets::{models, requests, responses};
use crate::extractors::{ApiToken, DataStore, DbConn, JsonMultipartUpload};

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
        r#"INSERT INTO buckets (account_id, friendly_name, type) VALUES ($1, $2, $3) RETURNING id;"#,
        api_token.subject,
        new_bucket.friendly_name,
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

        friendly_name: new_bucket.friendly_name,
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
            friendly_name: "test interactive bucket".to_string(),
            r#type: responses::BucketType::Interactive,
        },
        responses::MinimalBucket {
            id: "7bce1c56-71b9-4147-80d4-7519a7e98bd3".to_string(),
            friendly_name: "test backup bucket".to_string(),
            r#type: responses::BucketType::Backup,
        },
    ];

    (StatusCode::OK, axum::Json(bucket_list)).into_response()
}

#[derive(Debug)]
enum MetadataUploadState {
    WaitingForDetails,
    WaitingForUpload,
}

pub async fn publish_metadata(
    _api_token: ApiToken,
    Path(bucket_id): Path<Uuid>,
    //_if_match: Option<TypedHeader<IfMatch>>,
    store: DataStore,
) -> Response {
    // todo: authorization
    // todo: If-Match matches existing version abort

    //while let Some(res_field) = multipart.next_field().await.transpose() {
    //    let field = match res_field {
    //        Ok(f) => f,
    //        Err(err) => {
    //            return (StatusCode::BAD_REQUEST, format!("unknown error receiving field: {err}")).into_response();
    //        }
    //    };

    //    let content_type = match field.content_type() {
    //        Some(ct) => ct,
    //        None => {
    //            return (StatusCode::BAD_REQUEST, "multipart fields must contain content types").into_response();
    //        }
    //    };

    //    match upload_state {
    //        MetadataUploadState::WaitingForDetails => {
    //            if content_type != "application/json" {
    //                return (StatusCode::BAD_REQUEST, "first field must be JSON").into_response();
    //            }

    //            // todo: process body as stream, limit the size of the 

    //            upload_state = MetadataUploadState::WaitingForUpload;
    //        }
    //        MetadataUploadState::WaitingForUpload => {
    //            let file_name = format!("{bucket_id}/{}.car", Uuid::new_v4());
    //            let file_path = object_store::path::Path::from(file_name.as_str());

    //            let (upload_id, mut writer) = match store.put_multipart(&file_path).await {
    //                Ok(mp) => mp,
    //                Err(_err) => {
    //                    return (
    //                        StatusCode::INTERNAL_SERVER_ERROR,
    //                        "unable to store uploaded file",
    //                        )
    //                        .into_response();
    //                }
    //            };

    //            let file_hash = match handle_metadata_upload(&mut writer, stream).await {
    //                Ok(fh) => {
    //                    writer
    //                        .shutdown()
    //                        .await
    //                        .expect("upload finalization to succeed");
    //                    fh
    //                }
    //                Err(_err) => {
    //                    store
    //                        .abort_multipart(&file_path, &upload_id)
    //                        .await
    //                        .expect("aborting to success");
    //                    return (
    //                        StatusCode::INTERNAL_SERVER_ERROR,
    //                        "unable to process upload",
    //                        )
    //                        .into_response();
    //                }
    //            };

    //            return (StatusCode::OK, file_hash).into_response();
    //        }
    //    }
    //}

    (StatusCode::OK, "placeholder").into_response()
}

//async fn handle_metadata_upload(
//    writer: &mut Box<dyn AsyncWrite + Unpin + Send>,
//    mut stream: BodyStream,
//) -> Result<String, ()> {
//    let mut car_buffer = CarBuffer::new();
//    let mut hasher = blake3::Hasher::new();
//
//    while let Some(chunk) = stream.try_next().await.transpose() {
//        let chunk = chunk.expect("an available chunk (todo remove this)");
//
//        hasher.update(&chunk);
//        car_buffer.add_chunk(&chunk);
//
//        match car_buffer.parse() {
//            Ok(Some(_md)) => {
//                // todo: we have our metadata, do any validation we need to here
//            }
//            Ok(None) => (),
//            Err(err) => {
//                tracing::error!("received car buffer error: {err}");
//                return Err(());
//            }
//        }
//
//        writer
//            .write_all(&chunk)
//            .await
//            .expect("the write to succeed (todo remove this)");
//    }
//
//    let hash = hasher.finalize();
//
//    Ok(hash.to_string())
//}

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
        friendly_name: "test interactive bucket".to_string(),
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
