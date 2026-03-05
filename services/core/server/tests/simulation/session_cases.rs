use http::StatusCode;
use server::{
    dtos::session_dto::SessionDTO,
    endpoints::session_endpoints::{GetSessionRequest, GetSessionTag, SESSION_ENDPOINT},
};

use crate::shared::case::TestSuite;

pub async fn register(suite: &mut TestSuite, token: &str) {
    test_group!(suite, "sessions", {
        suite.case("get current session")
            .get(SESSION_ENDPOINT)
            .query(GetSessionRequest::Default { tag: GetSessionTag::Current })
            .token(token)
            .expect(|r| {
                assert_eq!(r.status_code(), StatusCode::OK);
                let sessions: Vec<SessionDTO> = r.json();
                assert_eq!(sessions.len(), 1);
            })
            .await;
    });
}
