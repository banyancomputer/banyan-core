use serde::Serialize;

#[derive(Serialize)]
pub struct WhoAmI {
    pub account_id: String,
}
