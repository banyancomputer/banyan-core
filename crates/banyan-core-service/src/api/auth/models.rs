use sqlx::FromRow;

#[derive(FromRow)]
pub struct CreatedAccount {
    pub id: String,
}

#[derive(FromRow)]
pub struct CreatedDeviceKey {
    pub id: String,
}
