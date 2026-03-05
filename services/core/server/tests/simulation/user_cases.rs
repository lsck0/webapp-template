use http::StatusCode;
use serde_json::Value;
use server::endpoints::{
    auth_endpoints::{PasswordChangeInfo, LOGOUT_ENDPOINT, PASSWORD_CHANGE_ENDPOINT},
    user_endpoints::{UpdateUserRequest, USER_ENDPOINT},
};

use crate::shared::case::TestSuite;

pub async fn register(suite: &mut TestSuite, token: &str) {
    test_group!(suite, "users", {
        suite.case("get user")
            .get(USER_ENDPOINT)
            .token(token)
            .expect(|r| assert_eq!(r.status_code(), StatusCode::OK))
            .await;

        suite.case("update name")
            .put(USER_ENDPOINT)
            .body(UpdateUserRequest {
                name: "alice_renamed".into(),
            })
            .token(token)
            .expect(|r| {
                assert_eq!(r.status_code(), StatusCode::OK);
                let body: Value = r.json();
                assert_eq!(body["name"], "alice_renamed");
            })
            .await;
    });

    test_group!(suite, "password change", {
        suite.case("change password")
            .post(PASSWORD_CHANGE_ENDPOINT)
            .body(PasswordChangeInfo {
                old_password: "alice_pass_1234".into(),
                new_password: "alice_new_pass".into(),
                otp: None,
            })
            .token(token)
            .expect(|r| assert_eq!(r.status_code(), StatusCode::OK))
            .await;
    });

    test_group!(suite, "logout", {
        suite.case("logout")
            .post(LOGOUT_ENDPOINT)
            .token(token)
            .expect(|r| assert_eq!(r.status_code(), StatusCode::OK))
            .await;

        suite.case("logout again")
            .post(LOGOUT_ENDPOINT)
            .token(token)
            .expect(|r| assert_eq!(r.status_code(), StatusCode::UNAUTHORIZED))
            .await;
    });
}
