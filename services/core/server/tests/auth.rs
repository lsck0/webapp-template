use auth::RegisterRequest;
use config::ServerConfig;
use http::{Method, StatusCode};
use models::{DbInitFlags, models::user_model::UserModel};
use pretty_assertions::assert_eq;
use serial_test::serial;
use server::{
    dtos::session_dto::SessionDTO,
    endpoints::{
        auth_endpoints::{
            INVITE_ENDPOINT, LOGIN_ENDPOINT, LOGOUT_ENDPOINT, LoginInfo, PASSWORD_CHANGE_ENDPOINT,
            PasswordChangeInfo, REGISTER_ENDPOINT,
        },
        session_endpoints::{GetSessionRequest, GetSessionTag, SESSION_ENDPOINT},
    },
};
use shared::{
    check_request, create_test_server, get_authed_client_for, get_authed_test_client, get_response,
    refresh_client_session,
};

mod shared;

#[tokio::test]
#[serial]
async fn auth_test_base() {
    let server = create_test_server(DbInitFlags::NUKE).await;
    let (client, session) = get_authed_test_client(DbInitFlags::NONE).await;

    check_request!(
        server,
        Method::GET,
        SESSION_ENDPOINT,
        QueryParams(GetSessionRequest::Default {
            tag: GetSessionTag::Current,
        }),
        StatusCode::UNAUTHORIZED
    );

    let responded_session = get_response!(
        client,
        Method::GET,
        SESSION_ENDPOINT,
        QueryParams(GetSessionRequest::Default {
            tag: GetSessionTag::Current,
        }),
        StatusCode::OK,
        Vec<SessionDTO>
    )[0]
    .clone();

    assert_eq!(responded_session.id, session.id);
}

#[tokio::test]
#[serial]
async fn auth_flow() {
    let server = create_test_server(DbInitFlags::NUKE).await;
    let (mut client, session) = get_authed_test_client(DbInitFlags::NONE).await;

    // register without an invite should fail
    let response = server
        .post(REGISTER_ENDPOINT)
        .json(&RegisterRequest {
            name: String::from("auth_flow"),
            password: String::from("auth_flow"),
            invite: String::from("auth_flow"),
        })
        .await;
    assert_eq!(response.status_code(), 422);

    // create an invite with owner
    let response = client.get(INVITE_ENDPOINT).await;
    assert_eq!(response.status_code(), 200);

    let invite = response.text();

    // create a user and log in
    let username = "auth_flow";
    let password = "auth_flow";
    let response = server
        .post(REGISTER_ENDPOINT)
        .json(&RegisterRequest {
            name: username.to_string(),
            password: password.to_string(),
            invite: invite.clone(),
        })
        .await;
    assert_eq!(response.status_code(), 200);

    let users = UserModel::get_all().unwrap();
    assert_eq!(users.len(), 2);

    let response = server
        .post(LOGIN_ENDPOINT)
        .json(&LoginInfo {
            name: username.to_string(),
            password: password.to_string(),
            otp: None,
        })
        .await;
    assert_eq!(response.status_code(), 200);

    // using the invite twice should fail
    let response = server
        .post(REGISTER_ENDPOINT)
        .json(&RegisterRequest {
            name: "auth_flow2".to_string(),
            password: "auth_flow2".to_string(),
            invite: invite.clone(),
        })
        .await;
    assert_eq!(response.status_code(), 422);

    // creating an invite with user permissions should fail
    let (user_client, _) = get_authed_client_for(username, password, DbInitFlags::NONE).await;

    let response = user_client.get(INVITE_ENDPOINT).await;
    assert_eq!(response.status_code(), 403);

    // changing the password should work
    let new_password = "new_password";
    let response = user_client
        .post(PASSWORD_CHANGE_ENDPOINT)
        .json(&PasswordChangeInfo {
            old_password: password.to_string(),
            new_password: new_password.to_string(),
            otp: None,
        })
        .await;
    assert_eq!(response.status_code(), 200);

    let response = server
        .post(LOGIN_ENDPOINT)
        .json(&LoginInfo {
            name: username.to_string(),
            password: new_password.to_string(),
            otp: None,
        })
        .await;
    assert_eq!(response.status_code(), 200);

    // logging out should work
    let response = user_client.post(LOGOUT_ENDPOINT).await;
    assert_eq!(response.status_code(), 200);

    // logging out again should fail
    let response = user_client.post(LOGOUT_ENDPOINT).await;
    assert_eq!(response.status_code(), 401);

    // logging with the wrong password should fail
    let response = server
        .post(LOGIN_ENDPOINT)
        .json(&LoginInfo {
            name: username.to_string(),
            password: "wrong_password".to_string(),
            otp: None,
        })
        .await;
    assert_eq!(response.status_code(), 422);

    // repeatedly trying the wrong password should lock the account
    for _ in 0..=ServerConfig::get().max_login_attempts {
        let response = server
            .post(LOGIN_ENDPOINT)
            .json(&LoginInfo {
                name: username.to_string(),
                password: "wrong_password".to_string(),
                otp: None,
            })
            .await;
        assert_eq!(response.status_code(), 422);
    }

    let response = server
        .post(LOGIN_ENDPOINT)
        .json(&LoginInfo {
            name: username.to_string(),
            password: new_password.to_string(),
            otp: None,
        })
        .await;
    assert_eq!(response.status_code(), 422);

    // revalidating the account should work
    let session = refresh_client_session(&mut client, session).await;

    let response = client
        .get(SESSION_ENDPOINT)
        .add_query_params(GetSessionRequest::Default {
            tag: GetSessionTag::Current,
        })
        .await;
    assert_eq!(response.status_code(), 200);

    let responded_session = response.json::<Vec<SessionDTO>>()[0].clone();

    assert_eq!(responded_session.id, session.id);

    // log this user out as well
    let response = client.post(LOGOUT_ENDPOINT).await;
    assert_eq!(response.status_code(), 200);
}
