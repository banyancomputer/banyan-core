pub async fn push(
    api_token: ApiToken,
    database: Database,
    store: DataStore,
    signing_key: SigningKey,
    Path(bucket_id): Path<Uuid>,
    TypedHeader(content_type): TypedHeader<ContentType>,
    body: BodyStream,
) -> impl IntoResponse {
    // ...

    let storage_authorization = match generate_storage_ticket(
        &account_id,
        &storage_grant_id,
        api_token_kid,
        &storage_host.name,
        &storage_host.url,
        data_usage,
        &signing_key,
    ) {
        Ok(ticket) => ticket,
        Err(err) => {
            return CoreError::default_error(&format!("unable to generate storage ticket: {err}"))
                .into_response();
        }
    };

}

/// Handle a request to pull metadata from a bucket
pub async fn pull(
    api_token: ApiToken,
    database: Database,
    store: DataStore,
    Path((bucket_id, metadata_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    let account_id = api_token.subject;
    let bucket_id = bucket_id.to_string();
    let metadata_id = metadata_id.to_string();
    match db::authorize_bucket(&account_id, &bucket_id, &database).await {
        Ok(_) => {}
        Err(err) => match err {
            sqlx::Error::RowNotFound => {
                return (StatusCode::NOT_FOUND, format!("bucket not found: {err}")).into_response();
            }
            _ => {
                return CoreError::default_error(&format!("unable to read bucket: {err}"))
                    .into_response();
            }
        },
    };
    // Make sure the metadata exists
    match db::authorize_metadata(&bucket_id, &metadata_id, &database).await {
        Ok(_) => {}
        Err(err) => match err {
            sqlx::Error::RowNotFound => {
                return (StatusCode::NOT_FOUND, format!("metadata not found: {err}"))
                    .into_response();
            }
            _ => {
                return CoreError::default_error(&format!("unable to read metadata: {err}"))
                    .into_response();
            }
        },
    };

    // Try opening the file for reading
    let file_name = format!(
        "{bucket_id}/{metadata_id}.car",
        bucket_id = bucket_id,
        metadata_id = metadata_id
    );
    let file_path = object_store::path::Path::from(file_name.as_str());
    let reader = match store.get(&file_path).await {
        Ok(r) => r,
        Err(err) => {
            return CoreError::default_error(&format!("unable to read metadata file: {err}"))
                .into_response();
        }
    };
    let stream = reader.into_stream();

    // Create the headers for the response
    let mut headers = HeaderMap::new();

    headers.insert(
        http::header::CONTENT_TYPE,
        HeaderValue::from_static("application/vnd.ipld.car; version=2"),
    );
    headers.insert(
        http::header::CONTENT_DISPOSITION,
        HeaderValue::from_str(format!("attachment; filename=\"{file_name}\"").as_str()).unwrap(),
    );

    let body = StreamBody::new(stream);

    (StatusCode::OK, headers, body).into_response()
}
