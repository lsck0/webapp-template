use auth::RegisterRequest;
use config::ServerConfig;
use http::StatusCode;
use server::{
    dtos::session_dto::SessionDTO,
    endpoints::auth_endpoints::{
        LoginInfo, INVITE_ENDPOINT, LOGIN_ENDPOINT, REGISTER_ENDPOINT,
    },
    endpoints::session_endpoints::{GetSessionRequest, GetSessionTag, SESSION_ENDPOINT},
};

use crate::shared::case::TestSuite;

#[allow(unused)]
pub struct AuthState {
    pub admin_token: String,
    pub alice_token: String,
}

pub async fn register(suite: &mut TestSuite) -> AuthState {
    let config = ServerConfig::get();

    test_group!(suite, "auth flow", {
        suite.case("unauth session access")
            .get(SESSION_ENDPOINT)
            .query(GetSessionRequest::Default { tag: GetSessionTag::Current })
            .expect(|r| assert_eq!(r.status_code(), StatusCode::UNAUTHORIZED))
            .await;

        let resp = suite.case("admin login")
            .post(LOGIN_ENDPOINT)
            .body(LoginInfo {
                name: config.default_user_name.clone(),
                password: config.default_user_password.clone(),
                otp: None,
            })
            .expect(|r| assert_eq!(r.status_code(), StatusCode::OK))
            .await;
        let admin: SessionDTO = resp.json();
        let admin_token = admin.access_token.clone();

        suite.case("register no invite")
            .post(REGISTER_ENDPOINT)
            .body(RegisterRequest {
                name: "fail_user".into(),
                password: "password1234".into(),
                invite: "bogus".into(),
            })
            .expect(|r| assert_eq!(r.status_code(), StatusCode::UNPROCESSABLE_ENTITY))
            .await;

        let resp = suite.case("create invite")
            .get(INVITE_ENDPOINT)
            .token(&admin_token)
            .expect(|r| assert_eq!(r.status_code(), StatusCode::OK))
            .await;
        let invite = resp.text();

        suite.case("register alice")
            .post(REGISTER_ENDPOINT)
            .body(RegisterRequest {
                name: "alice".into(),
                password: "alice_pass_1234".into(),
                invite: invite.clone(),
            })
            .expect(|r| assert_eq!(r.status_code(), StatusCode::OK))
            .await;

        suite.case("reuse invite")
            .post(REGISTER_ENDPOINT)
            .body(RegisterRequest {
                name: "bob".into(),
                password: "bob_pass_1234".into(),
                invite,
            })
            .expect(|r| assert_eq!(r.status_code(), StatusCode::UNPROCESSABLE_ENTITY))
            .await;

        let resp = suite.case("alice login")
            .post(LOGIN_ENDPOINT)
            .body(LoginInfo {
                name: "alice".into(),
                password: "alice_pass_1234".into(),
                otp: None,
            })
            .expect(|r| assert_eq!(r.status_code(), StatusCode::OK))
            .await;
        let alice: SessionDTO = resp.json();
        let alice_token = alice.access_token.clone();

        suite.case("wrong password")
            .post(LOGIN_ENDPOINT)
            .body(LoginInfo {
                name: "alice".into(),
                password: "wrong".into(),
                otp: None,
            })
            .expect(|r| assert_eq!(r.status_code(), StatusCode::UNPROCESSABLE_ENTITY))
            .await;

        AuthState { admin_token, alice_token }
    })
}
