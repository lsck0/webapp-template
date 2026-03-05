#![allow(clippy::needless_return)]

use http::StatusCode;
use models::DbInitFlags;
use pretty_assertions::assert_eq;
use serial_test::serial;
use server::{
    dtos::{session_dto::SessionDTO, user_dto::UserDTO},
    endpoints::{
        session_endpoints::{
            CloseSessionRequest, DeleteSessionRequest, GetSessionRequest, GetSessionTag, SESSION_ENDPOINT,
        },
        user_endpoints::{USER_ENDPOINT, UpdateUserRequest},
    },
};
use shared::{create_test_server, get_authed_client_for, get_authed_test_client, register_user};

mod shared;

// ── Session endpoints ───────────────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn get_current_session() {
    let _server = create_test_server(DbInitFlags::NUKE).await;
    let (client, session) = get_authed_test_client(DbInitFlags::NONE).await;

    let response = client
        .get(SESSION_ENDPOINT)
        .add_query_params(GetSessionRequest::Default {
            tag: GetSessionTag::Current,
        })
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let sessions = response.json::<Vec<SessionDTO>>();
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].id, session.id);
}

#[tokio::test]
#[serial]
async fn get_all_sessions() {
    let _server = create_test_server(DbInitFlags::NUKE).await;
    let (client, _) = get_authed_test_client(DbInitFlags::NONE).await;

    let response = client
        .get(SESSION_ENDPOINT)
        .add_query_params(GetSessionRequest::Default {
            tag: GetSessionTag::All,
        })
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let sessions = response.json::<Vec<SessionDTO>>();
    assert!(!sessions.is_empty());
}

#[tokio::test]
#[serial]
async fn get_valid_sessions() {
    let _server = create_test_server(DbInitFlags::NUKE).await;
    let (client, session) = get_authed_test_client(DbInitFlags::NONE).await;

    let response = client
        .get(SESSION_ENDPOINT)
        .add_query_params(GetSessionRequest::Default {
            tag: GetSessionTag::Valid,
        })
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let sessions = response.json::<Vec<SessionDTO>>();
    assert!(sessions.iter().all(|s| s.valid));
    assert!(sessions.iter().any(|s| s.id == session.id));
}

#[tokio::test]
#[serial]
async fn close_specific_session() {
    let _server = create_test_server(DbInitFlags::NUKE).await;
    let (admin, _) = get_authed_test_client(DbInitFlags::NONE).await;

    let server2 = create_test_server(DbInitFlags::NONE).await;
    register_user(&admin, &server2, "closesess", "password1234").await;

    // Login twice to create two sessions
    let (client1, session1) = get_authed_client_for("closesess", "password1234", DbInitFlags::NONE).await;
    let (_client2, session2) = get_authed_client_for("closesess", "password1234", DbInitFlags::NONE).await;

    // Close session2 from client1
    let response = client1
        .put(SESSION_ENDPOINT)
        .json(&CloseSessionRequest::Id {
            id: session2.id.to_string(),
        })
        .await;
    assert_eq!(response.status_code(), StatusCode::OK);

    // Verify only session1 is still valid
    let response = client1
        .get(SESSION_ENDPOINT)
        .add_query_params(GetSessionRequest::Default {
            tag: GetSessionTag::Valid,
        })
        .await;
    assert_eq!(response.status_code(), StatusCode::OK);
    let valid_sessions = response.json::<Vec<SessionDTO>>();
    assert!(valid_sessions.iter().any(|s| s.id == session1.id));
    assert!(!valid_sessions.iter().any(|s| s.id == session2.id));
}

#[tokio::test]
#[serial]
async fn close_all_sessions() {
    let _server = create_test_server(DbInitFlags::NUKE).await;
    let (admin, _) = get_authed_test_client(DbInitFlags::NONE).await;

    let server2 = create_test_server(DbInitFlags::NONE).await;
    register_user(&admin, &server2, "closeall", "password1234").await;

    let (client, _) = get_authed_client_for("closeall", "password1234", DbInitFlags::NONE).await;
    // Login again to create a second session
    let (_client2, _) = get_authed_client_for("closeall", "password1234", DbInitFlags::NONE).await;

    let response = client
        .put(SESSION_ENDPOINT)
        .json(&CloseSessionRequest::All {})
        .await;
    assert_eq!(response.status_code(), StatusCode::OK);
}

#[tokio::test]
#[serial]
async fn delete_specific_session() {
    let _server = create_test_server(DbInitFlags::NUKE).await;
    let (admin, _) = get_authed_test_client(DbInitFlags::NONE).await;

    let server2 = create_test_server(DbInitFlags::NONE).await;
    register_user(&admin, &server2, "delsess", "password1234").await;

    let (client, _session1) = get_authed_client_for("delsess", "password1234", DbInitFlags::NONE).await;
    let (_client2, session2) = get_authed_client_for("delsess", "password1234", DbInitFlags::NONE).await;

    // Delete session2
    let response = client
        .delete(SESSION_ENDPOINT)
        .add_query_params(DeleteSessionRequest::Id {
            id: session2.id.to_string(),
        })
        .await;
    assert_eq!(response.status_code(), StatusCode::OK);

    // session2 should be gone entirely
    let response = client
        .get(SESSION_ENDPOINT)
        .add_query_params(GetSessionRequest::Default {
            tag: GetSessionTag::All,
        })
        .await;
    assert_eq!(response.status_code(), StatusCode::OK);
    let all_sessions = response.json::<Vec<SessionDTO>>();
    assert!(!all_sessions.iter().any(|s| s.id == session2.id));
}

#[tokio::test]
#[serial]
async fn session_endpoints_require_auth() {
    let server = create_test_server(DbInitFlags::NUKE).await;

    let response = server
        .get(SESSION_ENDPOINT)
        .add_query_params(GetSessionRequest::Default {
            tag: GetSessionTag::Current,
        })
        .await;
    assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);
}

// ── User endpoints ──────────────────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn get_current_user() {
    let _server = create_test_server(DbInitFlags::NUKE).await;
    let (client, session) = get_authed_test_client(DbInitFlags::NONE).await;

    let response = client.get(USER_ENDPOINT).await;
    assert_eq!(response.status_code(), StatusCode::OK);

    let users = response.json::<Vec<UserDTO>>();
    assert_eq!(users.len(), 1);
    assert_eq!(users[0].id, session.user.id);
    assert_eq!(users[0].name, session.user.name);
}

#[tokio::test]
#[serial]
async fn update_user_name() {
    let _server = create_test_server(DbInitFlags::NUKE).await;
    let (admin, _) = get_authed_test_client(DbInitFlags::NONE).await;

    let server2 = create_test_server(DbInitFlags::NONE).await;
    register_user(&admin, &server2, "updtname", "password1234").await;
    let (client, _) = get_authed_client_for("updtname", "password1234", DbInitFlags::NONE).await;

    let response = client
        .put(USER_ENDPOINT)
        .json(&UpdateUserRequest {
            name: String::from("newname2"),
        })
        .await;
    assert_eq!(response.status_code(), StatusCode::OK);

    let updated = response.json::<UserDTO>();
    assert_eq!(updated.name, "newname2");
}

#[tokio::test]
#[serial]
async fn delete_user() {
    let _server = create_test_server(DbInitFlags::NUKE).await;
    let (admin, _) = get_authed_test_client(DbInitFlags::NONE).await;

    let server2 = create_test_server(DbInitFlags::NONE).await;
    register_user(&admin, &server2, "deleteme", "password1234").await;
    let (client, session) = get_authed_client_for("deleteme", "password1234", DbInitFlags::NONE).await;

    let response = client.delete(USER_ENDPOINT).await;
    assert_eq!(response.status_code(), StatusCode::OK);

    // User should no longer exist
    let user = models::models::user_model::UserModel::find_by_id(session.user.id).unwrap();
    assert!(user.is_none(), "user should be deleted");
}

#[tokio::test]
#[serial]
async fn user_endpoints_require_auth() {
    let server = create_test_server(DbInitFlags::NUKE).await;

    let response = server.get(USER_ENDPOINT).await;
    assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);

    let response = server
        .put(USER_ENDPOINT)
        .json(&UpdateUserRequest {
            name: String::from("hacker"),
        })
        .await;
    assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);

    let response = server.delete(USER_ENDPOINT).await;
    assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);
}
