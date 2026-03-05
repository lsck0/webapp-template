#![allow(clippy::needless_return)]

use std::{net::SocketAddr, str::FromStr};

use auth::{RefreshRequest, RegisterRequest};
use axum_test::TestServer;
use config::ServerConfig;
use http::{HeaderName, HeaderValue};
use models::{DbInitFlags, models::session_model::SessionModel};
use pretty_assertions::assert_eq;
use server::{
    dtos::session_dto::SessionDTO,
    endpoints::auth_endpoints::{LOGIN_ENDPOINT, LoginInfo, REFRESH_ENDPOINT, REGISTER_ENDPOINT},
};

#[allow(unused)]
pub mod macros;
#[allow(unused)]
pub use macros::*;

#[macro_use]
#[allow(unused)]
pub mod case;
#[allow(unused)]
pub mod fuzz;

/// Creates a test server.
#[allow(unused)]
pub async fn create_test_server(flags: DbInitFlags) -> TestServer {
    auth::keyring::KeyRing::close();
    models::close_db();

    TestServer::new(
        server::app(flags)
            .await
            .into_make_service_with_connect_info::<SocketAddr>(),
    )
    .expect("Error creating test server.")
}

/// Create an authenticated test server for a specific user.
#[allow(unused)]
pub async fn get_authed_client_for(name: &str, password: &str, flags: DbInitFlags) -> (TestServer, SessionDTO) {
    let mut server = create_test_server(flags).await;

    // login to get a session
    let login_response = server
        .post(LOGIN_ENDPOINT)
        .json(&LoginInfo {
            name: name.to_string(),
            password: password.to_string(),
            otp: None,
        })
        .await;

    assert_eq!(login_response.status_code(), 200);

    let session = login_response.json::<SessionDTO>().clone();
    let token = session.access_token.clone();

    // add the token to the headers for all future requests
    server.add_header(
        HeaderName::from_str("Authorization").expect("Error creating header name."),
        HeaderValue::from_str(&format!("Bearer {}", token)).expect("Error creating header value."),
    );

    return (server, session);
}

/// Create an authenticated test server for the default test user.
#[allow(unused)]
pub async fn get_authed_test_client(flags: DbInitFlags) -> (TestServer, SessionDTO) {
    let config = ServerConfig::get();
    return get_authed_client_for(&config.default_user_name, &config.default_user_password, flags).await;
}

/// Refresh the session of the client.
#[allow(unused)]
pub async fn refresh_client_session(server: &mut TestServer, session: SessionDTO) -> SessionDTO {
    // since we cannot remove the old auth header, we have to make a new one
    let mut new_server = create_test_server(DbInitFlags::NONE).await;

    // refresh the session
    let refresh_response = server
        .post(REFRESH_ENDPOINT)
        .json(&RefreshRequest {
            session_id: session.id,
            session_token: session.session_token.clone(),
        })
        .await;

    assert_eq!(refresh_response.status_code(), 200);

    let new_session = refresh_response.json::<SessionDTO>().clone();
    let new_token = new_session.access_token.clone();

    // sanity check
    let stored_session = SessionModel::find_by_id(session.id).unwrap().unwrap();
    assert_eq!(stored_session.access_token, new_token);

    // add the token to the headers for all future requests
    new_server.add_header(
        HeaderName::from_str("Authorization").expect("Error creating header name."),
        HeaderValue::from_str(&format!("Bearer {}", new_token)).expect("Error creating header value."),
    );

    *server = new_server;

    return new_session;
}

/// Registers a new user via invite. Requires an already-authenticated admin client to create the invite.
#[allow(unused)]
pub async fn register_user(admin_client: &TestServer, server: &TestServer, name: &str, password: &str) {
    let response = admin_client.get(server::endpoints::auth_endpoints::INVITE_ENDPOINT).await;
    assert_eq!(response.status_code(), 200);
    let invite = response.text();

    let response = server
        .post(REGISTER_ENDPOINT)
        .json(&RegisterRequest {
            name: name.to_string(),
            password: password.to_string(),
            invite,
        })
        .await;
    assert_eq!(response.status_code(), 200);
}
