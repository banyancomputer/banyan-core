use axum::Router;
use tower_http::services::{ServeDir, ServeFile};

pub fn router() -> Router {
    let static_file_handler = ServeDir::new("public")
        .append_index_html_on_directories(true)
        .not_found_service(ServeFile::new("public/404.html"));

    Router::new().fallback_service(static_file_handler)
}
