pub struct Subscription {
    id: String,

    price_key: String,
    title: String,

    stripe_product_id: Option<String>,

    allow_overage: bool,
    user_visible: bool,

    base_price: i32,
    storage_overage_price: i32,
    bandwidth_overage_price: i32,

    storage_allowance: i32,
    bandwidth_allowance: i32,
}
