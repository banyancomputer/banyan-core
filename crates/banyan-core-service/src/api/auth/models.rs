use sqlx::FromRow;

#[derive(FromRow)]
/// Emitted when user is created / logged in for the first time
/// id - unique row id in db 
pub struct CreatedAccount {
    pub id: String,
}

#[derive(FromRow)]
/// Emitted when a device is created  
/// id - unique row id in db
pub struct CreatedDeviceKey {
    pub id: String,
}
