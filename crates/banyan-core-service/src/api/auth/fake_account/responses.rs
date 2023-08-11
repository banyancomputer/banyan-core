use serde::{Deserialize, Serialize};
use validify::Validify;

#[derive(Clone, Debug, Serialize, Deserialize, Validify)]
pub struct CreateFakeAccount {
    pub id: String,
}
