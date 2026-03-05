#![allow(clippy::needless_return)]

use http::StatusCode;
use models::{DbInitFlags, models::user_model::UserModel};
use pretty_assertions::assert_eq;
use serial_test::serial;
use server::{
    dtos::{session_dto::SessionDTO, user_dto::UserDTO},
    endpoints::{
        auth_endpoints::{LOGIN_ENDPOINT, LoginInfo},
        user_endpoints::{USER_ENDPOINT, UpdateUserRequest},
    },
};
use shared::{create_test_server, get_authed_client_for, get_authed_test_client, register_user};

mod shared;

// ── Registration assigns name_id ────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn register_assigns_name_id() {
    let server = create_test_server(DbInitFlags::NUKE).await;
    let (admin, _) = get_authed_test_client(DbInitFlags::NONE).await;

    register_user(&admin, &server, "testuser", "password1234").await;

    let user = UserModel::find_by_name("testuser").unwrap().unwrap();
    assert!(user.name_id >= 0 && user.name_id < 10000, "name_id should be in range 0..10000");
}

#[tokio::test]
#[serial]
async fn register_multiple_same_name_get_different_ids() {
    let server = create_test_server(DbInitFlags::NUKE).await;
    let (admin, _) = get_authed_test_client(DbInitFlags::NONE).await;

    register_user(&admin, &server, "samename", "password1234").await;
    register_user(&admin, &server, "samename", "password5678").await;

    let users = UserModel::get_all()
        .unwrap()
        .into_iter()
        .filter(|u| u.name == "samename")
        .collect::<Vec<_>>();

    assert_eq!(users.len(), 2, "should have 2 users with the same name");
    assert_ne!(
        users[0].name_id, users[1].name_id,
        "same-name users must have different name_ids"
    );
}

// ── Login with tag ──────────────────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn login_with_name_tag() {
    let server = create_test_server(DbInitFlags::NUKE).await;
    let (admin, _) = get_authed_test_client(DbInitFlags::NONE).await;

    register_user(&admin, &server, "taglogin", "password1234").await;

    let user = UserModel::find_by_name("taglogin").unwrap().unwrap();
    let tag = user.tag();

    let response = server
        .post(LOGIN_ENDPOINT)
        .json(&LoginInfo {
            name: tag.clone(),
            password: String::from("password1234"),
            otp: None,
        })
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let session = response.json::<SessionDTO>();
    assert_eq!(session.user.name, "taglogin");
    assert_eq!(session.user.name_id, user.name_id);
}

#[tokio::test]
#[serial]
async fn login_with_plain_name_fallback() {
    let server = create_test_server(DbInitFlags::NUKE).await;
    let (admin, _) = get_authed_test_client(DbInitFlags::NONE).await;

    register_user(&admin, &server, "plainlogin", "password1234").await;

    // Login with just the name (no #id)
    let response = server
        .post(LOGIN_ENDPOINT)
        .json(&LoginInfo {
            name: String::from("plainlogin"),
            password: String::from("password1234"),
            otp: None,
        })
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let session = response.json::<SessionDTO>();
    assert_eq!(session.user.name, "plainlogin");
}

#[tokio::test]
#[serial]
async fn login_with_wrong_tag_fails() {
    let server = create_test_server(DbInitFlags::NUKE).await;
    let (admin, _) = get_authed_test_client(DbInitFlags::NONE).await;

    register_user(&admin, &server, "wrongtag", "password1234").await;

    let user = UserModel::find_by_name("wrongtag").unwrap().unwrap();
    // Use a different name_id
    let wrong_id = if user.name_id == 0 { 1 } else { 0 };
    let wrong_tag = format!("wrongtag#{}", wrong_id);

    let response = server
        .post(LOGIN_ENDPOINT)
        .json(&LoginInfo {
            name: wrong_tag,
            password: String::from("password1234"),
            otp: None,
        })
        .await;

    assert_eq!(response.status_code(), StatusCode::UNPROCESSABLE_ENTITY);
}

// ── UserDTO includes name_id ────────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn user_dto_includes_name_id() {
    let _server = create_test_server(DbInitFlags::NUKE).await;
    let (client, session) = get_authed_test_client(DbInitFlags::NONE).await;

    // The session's user DTO should have a name_id
    assert!(
        session.user.name_id >= 0 && session.user.name_id < 10000,
        "admin user should have valid name_id"
    );

    // GET /user should also include name_id
    let response = client.get(USER_ENDPOINT).await;
    assert_eq!(response.status_code(), StatusCode::OK);
    let users = response.json::<Vec<UserDTO>>();
    assert_eq!(users.len(), 1);
    assert_eq!(users[0].name_id, session.user.name_id);
}

// ── Update user name reassigns name_id ──────────────────────────────────────

#[tokio::test]
#[serial]
async fn update_user_name_reassigns_name_id() {
    let _server = create_test_server(DbInitFlags::NUKE).await;
    let (admin, _) = get_authed_test_client(DbInitFlags::NONE).await;

    let server2 = create_test_server(DbInitFlags::NONE).await;
    register_user(&admin, &server2, "changeme", "password1234").await;
    let (user_client, _) = get_authed_client_for("changeme", "password1234", DbInitFlags::NONE).await;

    let old_user = UserModel::find_by_name("changeme").unwrap().unwrap();

    // Update name
    let response = user_client
        .put(USER_ENDPOINT)
        .json(&UpdateUserRequest {
            name: String::from("newname1"),
        })
        .await;
    assert_eq!(response.status_code(), StatusCode::OK);

    let updated = response.json::<UserDTO>();
    assert_eq!(updated.name, "newname1");
    assert!(updated.name_id >= 0 && updated.name_id < 10000);

    // DB should reflect the change
    let db_user = UserModel::find_by_id(old_user.id).unwrap().unwrap();
    assert_eq!(db_user.name, "newname1");
    assert_eq!(db_user.name_id, updated.name_id);
}

#[tokio::test]
#[serial]
async fn update_user_same_name_keeps_name_id() {
    let _server = create_test_server(DbInitFlags::NUKE).await;
    let (admin, _) = get_authed_test_client(DbInitFlags::NONE).await;

    let server2 = create_test_server(DbInitFlags::NONE).await;
    register_user(&admin, &server2, "keepname", "password1234").await;
    let (user_client, _) = get_authed_client_for("keepname", "password1234", DbInitFlags::NONE).await;

    let old_user = UserModel::find_by_name("keepname").unwrap().unwrap();
    let old_name_id = old_user.name_id;

    // Update with same name
    let response = user_client
        .put(USER_ENDPOINT)
        .json(&UpdateUserRequest {
            name: String::from("keepname"),
        })
        .await;
    assert_eq!(response.status_code(), StatusCode::OK);

    let updated = response.json::<UserDTO>();
    assert_eq!(updated.name, "keepname");
    assert_eq!(updated.name_id, old_name_id, "name_id should not change when name stays the same");
}

// ── Tag formatting ──────────────────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn user_tag_format() {
    let _server = create_test_server(DbInitFlags::NUKE).await;
    let (admin, _) = get_authed_test_client(DbInitFlags::NONE).await;

    let server2 = create_test_server(DbInitFlags::NONE).await;
    register_user(&admin, &server2, "tagfmt", "password1234").await;

    let user = UserModel::find_by_name("tagfmt").unwrap().unwrap();
    let tag = user.tag();

    // Tag should be name#0000 format (zero-padded to 4 digits)
    assert!(tag.starts_with("tagfmt#"), "tag should start with name#");
    let id_part = &tag["tagfmt#".len()..];
    assert_eq!(id_part.len(), 4, "id part should be zero-padded to 4 digits");
    let parsed_id: i16 = id_part.parse().expect("id part should be numeric");
    assert_eq!(parsed_id, user.name_id);
}
