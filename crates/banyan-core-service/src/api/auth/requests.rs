use serde::Deserialize;
use validify::Validify;

#[derive(Clone, Debug, Deserialize, Validify)]
pub struct CreateAccount {
}
