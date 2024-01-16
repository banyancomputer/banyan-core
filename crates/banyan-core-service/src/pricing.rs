use std::sync::OnceLock;

use serde::Deserialize;

use crate::database::DatabaseConnection;

const CURRENT_PRICING_DATA: &[u8] = include_bytes!("../dist/pricing.ron");

static CURRENT_PRICE_CONFIG: OnceLock<Vec<PricingTier>> = OnceLock::new();

pub fn current_price_config() -> &'static [PricingTier] {
    CURRENT_PRICE_CONFIG.get_or_init(|| {
        ron::de::from_bytes(CURRENT_PRICING_DATA).unwrap()
    })
}

pub async fn sync_pricing(_conn: &mut DatabaseConnection) -> Result<(), sqlx::Error> {
    todo!()
}

#[derive(Debug, Deserialize)]
pub struct PricingTier {
    key: String,
    title: String,

    allow_overage: bool,
    user_visible: bool,

    price: Option<Price>,

    storage_allowance: usize,
    bandwidth_allowance: usize,
}

#[derive(Debug, Deserialize)]
pub struct Price {
    base: usize,
    storage_overage: usize,
    bandwidth_overage: usize,
}
