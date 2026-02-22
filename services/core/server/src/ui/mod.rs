mod components;
mod pages;

use axum::{Router, routing::get};

pub fn ui_router() -> Router {
    return Router::new().nest(
        "/api/ui",
        Router::new()
            .route("/", get(pages::index::index_html))
            .route("/{*uri}", get(pages::not_found::not_found_html)),
    );
}
