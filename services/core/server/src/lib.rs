#![allow(clippy::needless_return)]
#![feature(duration_constructors, trivial_bounds)]
#![forbid(clippy::unwrap_used)]

pub mod dtos;
pub mod endpoints;
pub mod extractors;
pub mod metrics;
pub mod middlewares;
pub mod tasks;
pub mod ui;

use auth::initialize_auth;
use axum::{Router, routing::get};
use axum_prometheus::PrometheusMetricLayer;
use middlewares::{add_tracing_layer, initialize_tracing};
use models::{DbInitFlags, initialize_database};
use once_cell::sync::Lazy;
use openssl_probe::try_init_openssl_env_vars;
use tasks::initialize_cron_tasks;
use tower_http::cors::CorsLayer;
use utoipa::{
    Modify, OpenApi,
    openapi::security::{Http, HttpAuthScheme, SecurityScheme},
};
use utoipauto::utoipauto;

#[utoipauto(paths = "./src/endpoints")]
#[derive(OpenApi)]
#[openapi(
    info(title = "Webapp Template API"),
    modifiers(&SecurityAddon)
)]
struct ApiDoc;

/// Security addon for the API documentation to allow for token authentication.
struct SecurityAddon;
impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme("token", SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)))
        }
    }
}

pub async fn app(flags: DbInitFlags) -> Router {
    unsafe {
        if !try_init_openssl_env_vars() {
            panic!("Failed to initialize OpenSSL environment variables");
        }
    }

    initialize_tracing();
    initialize_database(flags);
    initialize_auth();
    initialize_cron_tasks().await;

    let (prometheus_layer, metric_handle) = PrometheusMetricLayer::pair();

    // /api/ -> API router
    // /api/ui -> Alternative UI router, either to replace react or development testing
    // /api/docs -> API docs
    // /api/metrics -> Prometheus metrics (outside access blocked by nginx)
    let mut app: Router = Router::new()
        .merge(endpoints::endpoint_router())
        .merge(ui::ui_router())
        .merge(utoipa_swagger_ui::SwaggerUi::new("/api/docs").url("/api/docs/openapi.json", ApiDoc::openapi()))
        .route("/api/metrics", get(|| async move { metric_handle.render() }))
        .layer(prometheus_layer)
        .layer(
            CorsLayer::new()
                .allow_methods([
                    http::Method::GET,
                    http::Method::POST,
                    http::Method::PUT,
                    http::Method::PATCH,
                    http::Method::DELETE,
                    http::Method::OPTIONS,
                ])
                .allow_headers([
                    http::header::AUTHORIZATION,
                    http::header::CONTENT_TYPE,
                    http::header::ORIGIN,
                ])
                .allow_credentials(true),
        );

    app = add_tracing_layer(app);

    return app;
}
