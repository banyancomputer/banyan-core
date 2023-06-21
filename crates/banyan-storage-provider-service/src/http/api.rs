use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Json, Router};
use http::header::{ACCEPT, AUTHORIZATION, ORIGIN};
use http::Method;
use tower_http::cors::{Any, CorsLayer};
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::validate_request::ValidateRequestHeaderLayer;

const REQUEST_BODY_LIMIT: usize = 65_535;

pub fn router() -> Router {
    // todo: going to need to add methods as I make use of other methods in the API. Ideally I
    // would have a wrapper method to allow per route method configuration or even better something
    // that inspected the route matches and applied the correct method config for that path...
    let cors_layer = CorsLayer::new()
        .allow_methods(vec![Method::GET, Method::DELETE, Method::POST, Method::PUT])
        .allow_headers(vec![AUTHORIZATION, ACCEPT, ORIGIN])
        // todo: add domain as a config option and make this configurable
        .allow_origin(Any)
        // todo: I think this only restricts cookies, check if this restricts authorization
        .allow_credentials(false);

    Router::new()
        // Note: This only checks Content-Length so only restricts 'good' clients. Consider it a
        // todo to limit this more in the future. See:
        // https://docs.rs/tower-http/latest/tower_http/limit/index.html
        .layer(RequestBodyLimitLayer::new(REQUEST_BODY_LIMIT))
        .layer(ValidateRequestHeaderLayer::accept("application/json"))
        .layer(cors_layer)
        .fallback(not_found_handler)
}

async fn not_found_handler() -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({"status": "not found"})),
    )
}
