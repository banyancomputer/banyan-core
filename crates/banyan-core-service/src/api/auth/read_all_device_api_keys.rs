
pub async fn handler(
    api_token: ApiToken,
    State(state): State<AppState>,
) -> Response {
    let account_id = api_token.subject;
    let maybe_device_keys = sqlx::query_as!(
        models::DeviceApiKey,
        r#"SELECT id, account_id, fingerprint, pem FROM device_api_keys WHERE account_id = $1;"#,
        account_id
    )
    .fetch_all(&database)
    .await;

    let device_keys = match maybe_device_keys {
        Ok(dks) => dks,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("unable to read device keys: {err}"),
            )
                .into_response();
        }
    };

    Json(responses::ReadDeviceApiKeys(
        device_keys
            .into_iter()
            .map(|dk| responses::ReadDeviceApiKey {
                id: dk.id,
                fingerprint: dk.fingerprint,
                pem: dk.pem,
            })
            .collect(),
    ))
    .into_response()
}
