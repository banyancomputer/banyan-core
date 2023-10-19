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
