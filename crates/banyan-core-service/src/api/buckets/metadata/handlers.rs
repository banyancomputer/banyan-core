use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use http::{HeaderMap, HeaderValue};
use uuid::Uuid;

use crate::extractors::ApiToken;


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

pub async fn destroy(
    _api_token: ApiToken,
    Path(_bucket_id): Path<Uuid>,
    Path(_metadata_id): Path<Uuid>,
) -> Response {
    (StatusCode::NO_CONTENT, ()).into_response()
}

pub async fn download(
    _api_token: ApiToken,
    Path(_bucket_id): Path<Uuid>,
    Path(metadata_id): Path<Uuid>,
) -> Response {
    let metadata_file_name = format!("{metadata_id}.car");

    let mut headers = HeaderMap::new();

    headers.insert(
        http::header::CONTENT_TYPE,
        HeaderValue::from_static("application/vnd.ipld.car; version=2"),
    );
    headers.insert(
        http::header::CONTENT_DISPOSITION,
        HeaderValue::from_str(format!("attachment; filename=\"{metadata_file_name}\"").as_str())
            .unwrap(),
    );

    (StatusCode::OK, headers, "<metadata car file>").into_response()
}

pub async fn index(_api_token: ApiToken, Path(_bucket_id): Path<Uuid>) -> Response {
    let response = serde_json::json!([
        { "id": "e627f0cc-1cfc-42fb-a8cb-23a57edc4594", "metadata_size": 1_187, "state": "pending" },
        { "id": "8d834721-c707-41cb-937e-ccbf5eb2102a", "metadata_size": 41_378, "state": "current" },
        { "id": "4b35955f-8a10-4b97-b9d3-857fde03106a", "metadata_size": 41_378, "state": "snapshot" },
    ]);

    (StatusCode::OK, axum::Json(response)).into_response()
}

pub async fn show(
    _api_token: ApiToken,
    Path(_bucket_id): Path<Uuid>,
    Path(metadata_id): Path<Uuid>,
) -> Response {
    let response = serde_json::json!({
        "id": metadata_id,
        "state": "pending",

        "data_size": 1_567_120,
        "metadata_size": 78_120,

        "published_at": "20230804T171200+Z",

        "authorized_public_keys": [
            "98:01:73:9d:aa:e4:4e:c5:29:3d:4e:1f:53:d3:f4:d2:d4:26:d9:1c",
        ],
        "storage_providers": [
            "http://127.0.0.1:3002",
        ],
    });

    (StatusCode::OK, axum::Json(response)).into_response()
}

pub async fn snapshot(
    _api_token: ApiToken,
    Path(_bucket_id): Path<Uuid>,
    Path(_metadata_id): Path<Uuid>,
) -> Response {
    (StatusCode::UNAUTHORIZED, ()).into_response()
}
