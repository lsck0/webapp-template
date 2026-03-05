use std::net::SocketAddr;

use auth::{LoginRequest, RegisterRequest, RefreshRequest};
use axum::{
    Json, Router,
    extract::ConnectInfo,
    middleware,
    response::IntoResponse,
    routing::{get, post},
};
use errors::{ServerResult, UserError};
use http::{HeaderMap, header::USER_AGENT};
use ipnetwork::IpNetwork;
use models::models::permissions::Permissions;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;

use crate::{
    dtos::{DTO, Model, session_dto::SessionDTO, user_dto::UserDTO},
    middlewares::authentication_middleware,
};

pub const REGISTER_ENDPOINT: &str = "/api/auth/register";
pub const LOGIN_ENDPOINT: &str = "/api/auth/login";
pub const LOGOUT_ENDPOINT: &str = "/api/auth/logout";
pub const REFRESH_ENDPOINT: &str = "/api/auth/refresh";
pub const PASSWORD_CHANGE_ENDPOINT: &str = "/api/auth/password-change";
pub const INVITE_ENDPOINT: &str = "/api/auth/invite";
pub const OTP_ENABLE_ENDPOINT: &str = "/api/auth/otp/enable";
pub const OTP_DISABLE_ENDPOINT: &str = "/api/auth/otp/disable";
pub const OTP_VALIDATE_ENDPOINT: &str = "/api/auth/otp/validate";
pub const OTP_BACKUP_CODES_ENDPOINT: &str = "/api/auth/otp/backup-codes";

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
                    .route("/otp/enable", post(otp_enable_handler))
                    .route("/otp/validate", post(otp_validate_handler))
                    .route("/otp/disable", post(otp_disable_handler))
                    .route("/otp/backup-codes", post(otp_backup_codes_handler))
                    .layer(middleware::from_fn(authentication_middleware)),
            ),
    )
}

// ── Register ────────────────────────────────────────────────────────────────

/// Register a new user.
///
/// Auth: Not Required
/// Permissions: None
#[utoipa::path(
    post,
    path = REGISTER_ENDPOINT,
    request_body = RegisterRequest,
    responses(
        (status = 200, description = "Ok."),
        (status = 401, description = "Unauthenticated."),
        (status = 403, description = "Unauthorized."),
        (status = 422, description = "User Error.", body = UserError),
    ),
)]
async fn register_handler(Json(request): Json<RegisterRequest>) -> ServerResult<impl IntoResponse> {
    auth::register(request)?;

    return Ok(());
}

// ── Login ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[ts(export)]
pub struct LoginInfo {
    pub name: String,
    pub password: String,
    pub otp: Option<String>,
}

/// Login a user.
///
/// Auth: Not Required
/// Permissions: None
#[utoipa::path(
    post,
    path = LOGIN_ENDPOINT,
    request_body = LoginInfo,
    responses(
        (status = 200, description = "Ok.", body = SessionDTO),
        (status = 401, description = "Unauthenticated."),
        (status = 403, description = "Unauthorized."),
        (status = 422, description = "User Error.", body = UserError),
    ),
)]
async fn login_handler(
    headers: HeaderMap,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(login_info): Json<LoginInfo>,
) -> ServerResult<impl IntoResponse> {
    let session = auth::login(LoginRequest {
        name: login_info.name,
        password: login_info.password,
        otp: login_info.otp,
        ip_address: IpNetwork::from(addr.ip()),
        user_agent: headers
            .get(USER_AGENT)
            .and_then(|v| v.to_str().ok())
            .map_or(String::from("unknown"), |v| v.to_string()),
    })?;

    let session = session.to_dto()?;

    return Ok(Json(session));
}

// ── Logout ──────────────────────────────────────────────────────────────────

/// Logout a user.
///
/// Auth: Required
/// Permissions: None
#[utoipa::path(
    post,
    path = LOGOUT_ENDPOINT,
    security(("token" = [])),
    responses(
        (status = 200, description = "Ok."),
        (status = 401, description = "Unauthenticated."),
        (status = 403, description = "Unauthorized."),
        (status = 422, description = "User Error.", body = UserError),
    ),
)]
async fn logout_handler(session: SessionDTO) -> ServerResult<impl IntoResponse> {
    auth::logout(session.to_model()?)?;

    return Ok(());
}

// ── Refresh ─────────────────────────────────────────────────────────────────

/// Refresh a session.
///
/// Auth: Not Required
/// Permissions: None
#[utoipa::path(
    post,
    path = REFRESH_ENDPOINT,
    request_body = RefreshRequest,
    responses(
        (status = 200, description = "Ok.", body = SessionDTO),
        (status = 401, description = "Unauthenticated."),
        (status = 403, description = "Unauthorized."),
        (status = 422, description = "User Error.", body = UserError),
    ),
)]
async fn refresh_handler(Json(request): Json<RefreshRequest>) -> ServerResult<impl IntoResponse> {
    let session = auth::refresh(request)?;

    let session = session.to_dto()?;

    return Ok(Json(session));
}

// ── Password Change ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[ts(export)]
pub struct PasswordChangeInfo {
    pub old_password: String,
    pub new_password: String,
    pub otp: Option<String>,
}

/// Change the password of user belonging to the session.
///
/// Auth: Required
/// Permissions: None
#[utoipa::path(
    post,
    path = PASSWORD_CHANGE_ENDPOINT,
    request_body = PasswordChangeInfo,
    security(("token" = [])),
    responses(
        (status = 200, description = "Ok."),
        (status = 401, description = "Unauthenticated."),
        (status = 403, description = "Unauthorized."),
        (status = 422, description = "User Error.", body = UserError),
    ),
)]
async fn password_change_handler(
    session: SessionDTO,
    Json(pw_change_info): Json<PasswordChangeInfo>,
) -> ServerResult<impl IntoResponse> {
    auth::change_password(auth::PasswordChangeRequest {
        session_user_id: session.user.id,
        session_id: session.id,
        old_password: pw_change_info.old_password,
        new_password: pw_change_info.new_password,
        otp: pw_change_info.otp,
    })?;

    return Ok(());
}

// ── Invite ──────────────────────────────────────────────────────────────────

/// Create an invite to allow a new user to register.
///
/// Auth: Required
/// Permissions: CanInvite
#[utoipa::path(
    get,
    path = INVITE_ENDPOINT,
    security(("token" = [])),
    responses(
        (status = 200, description = "Ok.", body = String),
        (status = 401, description = "Unauthenticated."),
        (status = 403, description = "Unauthorized."),
        (status = 422, description = "User Error.", body = UserError),
    ),
)]
async fn invite_handler(user: UserDTO) -> ServerResult<impl IntoResponse> {
    auth::require_permission(user.id, Permissions::CanInvite)?;

    let token = auth::create_invite(user.id)?;

    return Ok(token);
}

// ── OTP ─────────────────────────────────────────────────────────────────────

/// Enable OTP for the current user.
/// Returns the TOTP secret, otpauth URL, QR code (data URI), and recovery codes.
/// OTP is not enforced until validated with `otp_validate`.
///
/// Auth: Required
/// Permissions: None
#[utoipa::path(
    post,
    path = OTP_ENABLE_ENDPOINT,
    security(("token" = [])),
    responses(
        (status = 200, description = "Ok.", body = auth::OtpEnableResponse),
        (status = 401, description = "Unauthenticated."),
        (status = 422, description = "User Error.", body = UserError),
    ),
)]
async fn otp_enable_handler(user: UserDTO) -> ServerResult<impl IntoResponse> {
    let result = auth::otp_enable(user.id)?;

    return Ok(Json(result));
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[ts(export)]
pub struct OtpCodeRequest {
    pub code: String,
}

/// Validate OTP setup by providing a code from the authenticator app.
/// This confirms the user scanned the QR code and activates OTP enforcement.
///
/// Auth: Required
/// Permissions: None
#[utoipa::path(
    post,
    path = OTP_VALIDATE_ENDPOINT,
    security(("token" = [])),
    request_body = OtpCodeRequest,
    responses(
        (status = 200, description = "Ok."),
        (status = 401, description = "Unauthenticated."),
        (status = 422, description = "User Error.", body = UserError),
    ),
)]
async fn otp_validate_handler(user: UserDTO, Json(body): Json<OtpCodeRequest>) -> ServerResult<impl IntoResponse> {
    auth::otp_validate(user.id, &body.code)?;
    return Ok(());
}

/// Disable OTP for the current user. Requires a valid TOTP or recovery code.
///
/// Auth: Required
/// Permissions: None
#[utoipa::path(
    post,
    path = OTP_DISABLE_ENDPOINT,
    security(("token" = [])),
    request_body = OtpCodeRequest,
    responses(
        (status = 200, description = "Ok."),
        (status = 401, description = "Unauthenticated."),
        (status = 422, description = "User Error.", body = UserError),
    ),
)]
async fn otp_disable_handler(user: UserDTO, Json(body): Json<OtpCodeRequest>) -> ServerResult<impl IntoResponse> {
    auth::otp_disable(user.id, &body.code)?;
    return Ok(());
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[ts(export)]
pub struct OtpBackupCodesResponse {
    pub recovery_codes: Vec<String>,
}

/// Regenerate OTP recovery codes. Requires a valid TOTP code (not a recovery code).
///
/// Auth: Required
/// Permissions: None
#[utoipa::path(
    post,
    path = OTP_BACKUP_CODES_ENDPOINT,
    security(("token" = [])),
    request_body = OtpCodeRequest,
    responses(
        (status = 200, description = "Ok.", body = OtpBackupCodesResponse),
        (status = 401, description = "Unauthenticated."),
        (status = 422, description = "User Error.", body = UserError),
    ),
)]
async fn otp_backup_codes_handler(
    user: UserDTO,
    Json(body): Json<OtpCodeRequest>,
) -> ServerResult<impl IntoResponse> {
    let codes = auth::otp_regenerate_backup_codes(user.id, &body.code)?;
    return Ok(Json(OtpBackupCodesResponse { recovery_codes: codes }));
}
