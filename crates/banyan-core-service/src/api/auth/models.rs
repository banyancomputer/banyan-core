use sqlx::FromRow;

#[derive(Debug, FromRow)]
pub struct CreatedResource {
    pub id: String,
}

#[derive(Debug, FromRow)]
pub struct CreatedDeviceKey {
    pub id: String,
}
