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

    /* 4. Now that we know the size of metadata, Check if the upload exceeds the user's storage quota. If so, abort with 413 */

    // Read how metadata and data the use has in the current and pending states across all buckets
    let current_usage = match db::read_total_usage(&account_id, &database).await {
        Ok(usage) => usage,
        Err(err) => {
            return CoreError::default_error(&format!(
                "unable to read account storage usage: {err}"
            ))
            .into_response();
        }
    };

    // Based on how much stuff there planning on pushing, reject the upload if it would exceed the quota
    // Expected usage is their current usage plus the size of the metadata they're uploading plus the size of the data they want to upload to a host
    let expected_data_size = request_data.expected_data_size as u64;
    let expected_usage = current_usage + metadata_size + expected_data_size;

    if expected_usage > ACCOUNT_STORAGE_QUOTA {
        // Mark the upload as failed
        let maybe_failed_metadata_upload = sqlx::query!(
            r#"UPDATE metadata SET state = $1 WHERE id = $2;"#,
            models::MetadataState::UploadFailed,
            metadata_resource.id
        )
        .execute(&database)
        .await;
        match maybe_failed_metadata_upload {
            Ok(_) => {}
            Err(err) => {
                return CoreError::default_error(&format!(
                    "unable to mark metadata upload as failed: {}",
                    err
                ))
                .into_response();
            }
        };
        // Return the correct response based on the result of the update
        return (
            StatusCode::PAYLOAD_TOO_LARGE,
            format!(
                "account storage quota exceeded: {current_usage} + {request_size} > {ACCOUNT_STORAGE_QUOTA}",
                current_usage = current_usage,
                request_size = expected_data_size + metadata_size,
                ACCOUNT_STORAGE_QUOTA = ACCOUNT_STORAGE_QUOTA
            ),
        )
            .into_response();
    }

    /* 5. Ah yes! They can indeed store this data. Mark the upload as complete and put it in the appropriate state */

    // Check that the user is actually asking for more data in this request.
    // If not, update the metadata state to current and return a proper response
    // If so, update the metadata state to pending and continue
    if expected_data_size == 0 {
        let current_metadata_state = models::MetadataState::Current.to_string();
        let metadata_size = metadata_size as i64;
        let expected_data_size = expected_data_size as i64;
        let maybe_current_metadata = sqlx::query_as!(
            models::CreatedResource,
            r#"UPDATE metadata SET state = $1, metadata_size = $2, data_size = $3, metadata_hash = $4 WHERE id = $5 RETURNING id;"#,
            current_metadata_state,
            metadata_size,
            expected_data_size,
            metadata_hash,
            metadata_resource.id
        ).fetch_one(&database).await;
        let current_metadata = match maybe_current_metadata {
            Ok(cr) => cr,
            Err(err) => {
                return CoreError::default_error(&format!(
                    "unable to update bucket metadata after push: {err}"
                ))
                .into_response();
            }
        };
        // Set all current metadata to outdated, except for the one we just uploaded
        let outdated_metadata_state = models::MetadataState::Outdated.to_string();
        let maybe_outdated_metadata = sqlx::query!(
            r#"UPDATE metadata SET state = $1 WHERE bucket_id = $2 AND id != $3 AND state = $4;"#,
            outdated_metadata_state,
            bucket_id,
            metadata_resource.id,
            current_metadata_state
        )
        .execute(&database)
        .await;
        match maybe_outdated_metadata {
            Ok(_) => {
                return (
                    StatusCode::OK,
                    axum::Json(responses::PushMetadataResponse {
                        id: current_metadata.id.to_string(),
                        state: models::MetadataState::Current,
                        storage_host: None,
                        storage_authorization: None,
                    }),
                )
                    .into_response()
            }
            Err(err) => {
                return CoreError::default_error(&format!(
                    "unable to update bucket metadata after push: {err}"
                ))
                .into_response();
            }
        }
    }

    // OK, they're actually asking for more data. Update the metadata state to pending and continue
    let metadata_state = models::MetadataState::Pending.to_string();
    let metadata_size = metadata_size as i64;
    let maybe_updated_metadata = sqlx::query_as!(
        models::CreatedResource,
        r#"UPDATE metadata SET state = $1, metadata_size = $2, metadata_hash = $3 WHERE id = $4 RETURNING id;"#,
        metadata_state,
        metadata_size,
        metadata_hash,
        metadata_resource.id
    )
    .fetch_one(&database)
    .await;
    let updated_metadata = match maybe_updated_metadata {
        Ok(cr) => cr,
        Err(err) => {
            return CoreError::default_error(&format!(
                "unable to update bucket metadata after push: {err}"
            ))
            .into_response();
        }
    };

    /* 6. Determine a storage host we can point them too. Determine what they're expected usage on that host will be after upload */

    // Since we only have one storage host, this is easy
    // Query the database for the current and pending data usage for the account
    let data_usage = match db::read_total_data_usage(&account_id, &database).await {
        Ok(usage) => usage,
        Err(err) => {
            return CoreError::default_error(&format!("unable to read account data usage: {err}"))
                .into_response();
        }
    };

    // Round up to the nearest 100 MiB
    let data_usage = round_to_nearest_100_mib(data_usage);

    // Read a storage host from the database. We only have one right now, so this is easy
    let storage_host = match db::select_storage_host(&database).await {
        Ok(sh) => sh,
        Err(err) => {
            return CoreError::default_error(&format!("unable to read storage host: {err}"))
                .into_response();
        }
    };
    // TODO: Check if the storage host is full. If so, abort with 503

    /* 7. Generate a JWT for the storage host and return it to the user */
    let storage_grant_id = match db::record_storage_grant(
        &storage_host.id,
        &account_id,
        &metadata_resource.id,
        data_usage,
        &database,
    )
    .await
    {
        Ok(sgi) => sgi,
        Err(err) => {
            return CoreError::default_error(&format!("unable record storage grant: {err}"))
                .into_response();
        }
    };

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

    let response = responses::PushMetadataResponse {
        id: updated_metadata.id.to_string(),
        state: models::MetadataState::Pending,
        storage_host: Some(storage_host.url),
        storage_authorization: Some(storage_authorization),
    };

    (StatusCode::OK, axum::Json(response)).into_response()
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
