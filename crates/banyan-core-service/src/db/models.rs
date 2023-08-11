use sqlx::FromRow;

/// Catch all get id of created resource
#[derive(Debug, FromRow)]
pub struct CreatedResource {
    pub id: String,
}

/// DeviceApiKey
#[derive(Debug, FromRow)]
pub struct DeviceApiKey {
    pub id: String,
    pub account_id: String,
    pub fingerprint: String,
    pub pem: String,
}
