use axum::{
    Router, middleware,
    routing::{get, post},
};

use crate::{
    endpoints::auth_endpoints::{
        invite_endpoint::invite_handler, login_endpoint::login_handler, logout_endpoint::logout_handler,
        password_change_endpoint::password_change_handler, refresh_endpoint::refresh_handler,
        register_endpoint::register_handler,
    },
    middlewares::authentication_middleware,
};

pub mod invite_endpoint;
pub mod login_endpoint;
pub mod logout_endpoint;
pub mod otp_endpoints;
pub mod password_change_endpoint;
pub mod refresh_endpoint;
pub mod register_endpoint;

pub fn auth_router() -> Router {
    Router::new().nest(
        "/auth",
        Router::new()
            .route("/register", post(register_handler))
            .route("/login", post(login_handler))
            .route("/refresh", post(refresh_handler))
            .merge(
                Router::new()
                    .route("/invite", get(invite_handler))
                    .route("/logout", post(logout_handler))
                    .route("/password-change", post(password_change_handler))
                    .layer(middleware::from_fn(authentication_middleware)),
            ),
    )
}
