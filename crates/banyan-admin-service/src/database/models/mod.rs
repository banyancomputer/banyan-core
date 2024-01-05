mod deal;
mod deal_state;
mod user;

pub use deal::Deal;
pub use deal_state::DealState;
use jwt_simple::prelude::Serialize;
use jwt_simple::reexports::rand::Rng;
pub use user::User;

#[derive(sqlx::FromRow)]
pub struct ExplicitBigInt {
    big_int: i64,
}
