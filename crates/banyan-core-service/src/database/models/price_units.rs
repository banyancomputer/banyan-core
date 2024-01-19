use serde::{Deserialize, Serialize};

use crate::pricing::{PRICE_UNIT_TO_CENTS_RATE, PRICE_UNIT_TO_USD_RATE};

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize, sqlx::Type)]
#[serde(transparent)]
#[sqlx(transparent)]
pub struct PriceUnits(i64);

impl PriceUnits {
    pub fn new(val: i64) -> Self {
        Self(val)
    }

    pub fn in_cents(&self) -> i64 {
        self.0 / PRICE_UNIT_TO_CENTS_RATE as i64
    }

    pub fn in_fractional_cents(&self) -> String {
        format!("{:.12}", self.in_usd() * 100.)
    }

    pub fn in_usd(&self) -> f32 {
        self.0 as f32 / PRICE_UNIT_TO_USD_RATE as f32
    }
}
