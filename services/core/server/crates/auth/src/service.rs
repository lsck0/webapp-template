#![allow(clippy::needless_return)]

use std::collections::HashSet;

use chrono::{Duration, Utc};
use errors::{FieldError, FieldErrorReason, ServerError, ServerResult, bail, body_error, user_error};
use ipnetwork::IpNetwork;
use models::models::{
    invite_model::InviteModel,
    login_attempt_model::LoginAttemptModel,
    login_restriction_model::{LoginRestrictionModel, NewLoginRestrictionModel},
    permissions::Permissions,
    session_model::{NewSessionModel, SessionInvalidationReason, SessionModel},
    user_model::{NewUserModel, UserModel},
};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{HashedPassword, InviteToken, JWTToken, OTPBackupToken};

pub struct LoginRequest {
    pub name: String,
    pub password: String,
    pub otp: Option<String>,
    pub ip_address: IpNetwork,
    pub user_agent: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[ts(export)]
pub struct RegisterRequest {
    pub name: String,
    pub password: String,
    pub invite: String,
}

pub struct PasswordChangeRequest {
    pub session_user_id: Uuid,
    pub session_id: Uuid,
    pub old_password: String,
    pub new_password: String,
    pub otp: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[ts(export)]
pub struct RefreshRequest {
    #[schema(value_type = String)]
    pub session_id: Uuid,
    pub session_token: String,
}

/// Login a user and create a session.
pub fn login(request: LoginRequest) -> ServerResult<SessionModel> {
    return crate::defense::constant_time(|| {
        let config = config::ServerConfig::get();
        let mut errors = vec![];

        let Some(user) = parse_tag(&request.name)
            .and_then(|(name, name_id)| UserModel::find_by_tag(name, name_id).ok())
            .flatten()
            .or_else(|| UserModel::find_by_name(&request.name).ok().flatten())
        else {
            push_invalid_login_errors(&mut errors);
            bail!(body_error!(errors));
        };

        if let Some(restriction) = LoginRestrictionModel::find_by_user_id(user.id)?
            && restriction.restricted_until > Utc::now()
        {
            push_invalid_login_errors(&mut errors);
            bail!(body_error!(errors));
        }

        if !HashedPassword::parse(&user.password_hash)?.check(&request.password) {
            register_failed_login_attempt(&user.id)?;
            push_invalid_login_errors(&mut errors);
            bail!(body_error!(errors));
        }

        if user.otp_enabled && user.otp_validated {
            verify_otp(&user, request.otp.as_deref())?;
        }

        let open_sessions = SessionModel::find_valid_by_user(user.id)?.len();
        if open_sessions >= config.max_open_sessions {
            bail!(user_error!(TooManyOpenSessions));
        }

        let mut session = SessionModel::new(NewSessionModel {
            user_id: user.id,
            session_token: String::new(),
            access_token: String::new(),
            ip_address: request.ip_address,
            user_agent: request.user_agent,
        })?;

        session.session_token = JWTToken::new_session_token(&session.id).take();
        session.access_token = JWTToken::new_access_token(&session.id).take();
        let session = session.persist()?;

        LoginAttemptModel::delete_all_for_user(&user.id)?;

        return Ok(session);
    });
}

/// Register a new user.
pub fn register(request: RegisterRequest) -> ServerResult<()> {
    return crate::defense::constant_time(|| {
        let mut errors = vec![];

        if request.name.len() < 4 || request.name.len() > 32 {
            errors.push(FieldError {
                field: String::from("name"),
                reason: FieldErrorReason::InvalidRange { min: 4, max: 32 },
            });
        }

        if request.password.len() < 8 || request.password.len() > 32 {
            errors.push(FieldError {
                field: String::from("password"),
                reason: FieldErrorReason::InvalidRange { min: 8, max: 32 },
            });
        }

        if !errors.is_empty() {
            bail!(body_error!(errors));
        }

        let Some(invite) = InviteModel::find_by_token(&request.invite)? else {
            errors.push(FieldError {
                field: String::from("invite"),
                reason: FieldErrorReason::InvalidInviteToken,
            });
            bail!(body_error!(errors));
        };

        if invite.used || invite.expires_at < Utc::now() {
            errors.push(FieldError {
                field: String::from("invite"),
                reason: FieldErrorReason::ExpiredInviteToken,
            });
            bail!(body_error!(errors));
        }

        let password_hash = HashedPassword::hash(&request.password)?.take();
        let name_id = UserModel::assign_name_id(&request.name)?;
        let user = UserModel::new(NewUserModel {
            name: request.name,
            name_id,
            password_hash,
        })?;

        invite.use_invite(user.id)?;

        return Ok(());
    });
}

/// Logout the current session.
pub fn logout(session: SessionModel) -> ServerResult<()> {
    session.close(SessionInvalidationReason::UserLogout)?;
    return Ok(());
}

/// Refresh a session's tokens.
pub fn refresh(request: RefreshRequest) -> ServerResult<SessionModel> {
    let Some(mut session) = SessionModel::find_by_id(request.session_id)? else {
        bail!(ServerError::Unauthenticated);
    };

    if !session.valid {
        bail!(ServerError::Unauthenticated);
    }

    let session_token_payload = JWTToken::parse(&request.session_token).and_then(|token| token.decode());

    if matches!(session_token_payload, Err(ServerError::InvalidToken)) {
        bail!(ServerError::Unauthenticated);
    }

    if matches!(session_token_payload, Err(ServerError::ExpiredToken)) {
        session.close(SessionInvalidationReason::Expired)?;
        bail!(ServerError::ExpiredToken);
    }

    session_token_payload?;

    // suspicious activity, there should not be two valid session tokens
    if session.session_token != request.session_token {
        session.close(SessionInvalidationReason::SessionTokenLeak)?;
        bail!(ServerError::Unauthenticated);
    }

    session.session_token = JWTToken::new_session_token(&session.id).take();
    session.access_token = JWTToken::new_access_token(&session.id).take();
    session.last_used = Utc::now();
    let session = session.persist()?;

    return Ok(session);
}

/// Create an invite token.
pub fn create_invite(created_by: Uuid) -> ServerResult<String> {
    let token = InviteToken::create().take();
    let invite = InviteModel::new(created_by, token, Duration::days(1))?;
    return Ok(invite.token);
}

/// Change a user's password.
pub fn change_password(request: PasswordChangeRequest) -> ServerResult<()> {
    return crate::defense::constant_time(|| {
        let mut errors = vec![];

        let Some(mut user) = UserModel::find_by_id(request.session_user_id)? else {
            bail!(ServerError::Unauthenticated);
        };

        if !HashedPassword::parse(&user.password_hash)?.check(&request.old_password) {
            errors.push(FieldError {
                field: String::from("old_password"),
                reason: FieldErrorReason::InvalidCredentials,
            });
            bail!(body_error!(errors));
        }

        if user.otp_enabled && user.otp_validated {
            verify_otp(&user, request.otp.as_deref())?;
        }

        user.password_hash = HashedPassword::hash(&request.new_password)?.take();
        user.persist()?;

        // Invalidate all other sessions — the user must re-authenticate on other devices
        let sessions = SessionModel::find_valid_by_user(request.session_user_id)?;
        for session in sessions {
            if session.id != request.session_id {
                session.close(SessionInvalidationReason::PasswordChanged)?;
            }
        }

        return Ok(());
    });
}

/// Authenticate a request by validating an access token.
/// Not wrapped in constant_time: JWT validation has no timing side-channel
/// (no password comparison or user-existence leak).
pub fn authenticate(access_token: &str) -> ServerResult<SessionModel> {
    let session_id = JWTToken::parse(access_token)?.decode()?.subject_as_uuid()?;

    let Some(session) = SessionModel::find_by_id(session_id)? else {
        bail!(ServerError::Unauthenticated);
    };

    if !session.valid {
        bail!(ServerError::Unauthenticated);
    }

    // suspicious behavior, there should not be two valid access tokens
    if access_token != session.access_token {
        session.close(SessionInvalidationReason::AccessTokenLeak)?;
        bail!(ServerError::Unauthenticated);
    }

    return Ok(session);
}

fn push_invalid_login_errors(errors: &mut Vec<FieldError>) {
    errors.push(FieldError {
        field: String::from("name"),
        reason: FieldErrorReason::InvalidCredentials,
    });
    errors.push(FieldError {
        field: String::from("password"),
        reason: FieldErrorReason::InvalidCredentials,
    });
    errors.push(FieldError {
        field: String::from("otp"),
        reason: FieldErrorReason::InvalidCredentials,
    });
}

fn register_failed_login_attempt(user_id: &Uuid) -> ServerResult<()> {
    let config = config::ServerConfig::get();
    let past_attempt = LoginAttemptModel::find_past_attempt(user_id)?;

    match past_attempt {
        Some(mut attempt) => {
            attempt.failed_counter += 1;
            attempt.last_attempt = Utc::now();
            let attempt = attempt.persist()?;

            if attempt.failed_counter >= config.max_login_attempts {
                LoginRestrictionModel::new(NewLoginRestrictionModel {
                    user_id: *user_id,
                    restricted_until: Utc::now() + Duration::minutes(config.login_restriction_time),
                })?;
            }
        }
        None => {
            LoginAttemptModel::new(*user_id)?;
        }
    }

    return Ok(());
}

/// Resolve the effective permissions for a user.
pub fn resolve_permissions(user_id: Uuid) -> ServerResult<HashSet<Permissions>> {
    let Some(user) = UserModel::find_by_id(user_id)? else {
        bail!(ServerError::Unauthenticated);
    };
    return user.resolve_permissions();
}

/// Check that a user has the required permission. Returns `Err(Unauthorized)` if not.
pub fn require_permission(user_id: Uuid, required: Permissions) -> ServerResult<()> {
    let permissions = resolve_permissions(user_id)?;
    if !permissions.contains(&required) {
        bail!(ServerError::Unauthorized);
    }
    return Ok(());
}

/// Response for the OTP enable flow.
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[ts(export)]
pub struct OtpEnableResponse {
    pub secret: String,
    pub otpauth_url: String,
    pub qr_code_data_uri: String,
    pub recovery_codes: Vec<String>,
}

/// Begin OTP setup: generate secret, QR code, and recovery codes.
/// OTP is not enforced until `otp_validate` is called with a valid code.
pub fn otp_enable(user_id: Uuid) -> ServerResult<OtpEnableResponse> {
    let mut user = get_user(user_id)?;

    if user.otp_enabled && user.otp_validated {
        bail!(ServerError::UserError(errors::UserError::OtpAlreadyEnabled));
    }

    let (secret, url) = crate::totp::generate_secret(&user.name)?;

    let secret_bytes = totp_rs::Secret::Encoded(secret.clone())
        .to_bytes()
        .map_err(|_| ServerError::Internal("Bad secret".into()))?;
    let qr = crate::totp::generate_qr_data_uri(&user.name, &secret_bytes)?;

    let recovery_codes: Vec<String> = (0..8).map(|_| OTPBackupToken::create().take()).collect();

    user.otp_secret = Some(secret.clone());
    user.otp_url = Some(url.clone());
    user.otp_recovery_codes = Some(recovery_codes.iter().map(|c| Some(c.clone())).collect());
    user.otp_enabled = true;
    user.otp_validated = false;
    user.persist()?;

    return Ok(OtpEnableResponse {
        secret,
        otpauth_url: url,
        qr_code_data_uri: qr,
        recovery_codes,
    });
}

/// Validate OTP setup by verifying a TOTP code from the user's authenticator app.
/// This confirms the user has successfully scanned the QR code.
pub fn otp_validate(user_id: Uuid, code: &str) -> ServerResult<()> {
    let mut user = get_user(user_id)?;

    if !user.otp_enabled {
        bail!(ServerError::UserError(errors::UserError::OtpNotEnabled));
    }

    if user.otp_validated {
        bail!(ServerError::UserError(errors::UserError::OtpAlreadyEnabled));
    }

    let Some(ref secret) = user.otp_secret else {
        bail!(ServerError::Internal("OTP secret missing".into()));
    };

    if !crate::totp::verify(secret, code)? {
        bail!(ServerError::UserError(errors::UserError::InvalidOtp));
    }

    user.otp_validated = true;
    user.persist()?;

    return Ok(());
}

/// Disable OTP for a user. Requires a valid TOTP code or recovery code.
pub fn otp_disable(user_id: Uuid, code: &str) -> ServerResult<()> {
    let mut user = get_user(user_id)?;

    if !user.otp_enabled || !user.otp_validated {
        bail!(ServerError::UserError(errors::UserError::OtpNotEnabled));
    }

    verify_otp(&user, Some(code))?;

    user.otp_enabled = false;
    user.otp_validated = false;
    user.otp_secret = None;
    user.otp_url = None;
    user.otp_recovery_codes = None;
    user.persist()?;

    return Ok(());
}

/// Regenerate recovery codes for a user. Requires a valid TOTP code.
pub fn otp_regenerate_backup_codes(user_id: Uuid, code: &str) -> ServerResult<Vec<String>> {
    let mut user = get_user(user_id)?;

    if !user.otp_enabled || !user.otp_validated {
        bail!(ServerError::UserError(errors::UserError::OtpNotEnabled));
    }

    verify_otp_code_only(&user, code)?;

    let recovery_codes: Vec<String> = (0..8).map(|_| OTPBackupToken::create().take()).collect();
    user.otp_recovery_codes = Some(recovery_codes.iter().map(|c| Some(c.clone())).collect());
    user.persist()?;

    return Ok(recovery_codes);
}

/// Verify an OTP code or recovery code for a user.
fn verify_otp(user: &UserModel, code: Option<&str>) -> ServerResult<()> {
    let Some(code) = code else {
        bail!(ServerError::UserError(errors::UserError::OtpRequired));
    };

    let Some(ref secret) = user.otp_secret else {
        bail!(ServerError::Internal("OTP secret missing".into()));
    };

    // Try TOTP code first
    if crate::totp::verify(secret, code)? {
        return Ok(());
    }

    // Try recovery codes
    if let Some(ref codes) = user.otp_recovery_codes {
        for stored in codes.iter().flatten() {
            // Constant-time comparison to prevent timing attacks
            if stored.len() == code.len()
                && stored.bytes().zip(code.bytes()).fold(0u8, |acc, (a, b)| acc | (a ^ b)) == 0
            {
                // Consume the recovery code
                let mut user_mut = get_user(user.id)?;
                user_mut.otp_recovery_codes = Some(
                    codes
                        .iter()
                        .map(|c| match c {
                            Some(c) if c == code => None,
                            other => other.clone(),
                        })
                        .collect(),
                );
                user_mut.persist()?;
                return Ok(());
            }
        }
    }

    bail!(ServerError::UserError(errors::UserError::InvalidOtp));
}

/// Verify a TOTP code only (no recovery codes). Used for sensitive operations.
fn verify_otp_code_only(user: &UserModel, code: &str) -> ServerResult<()> {
    let Some(ref secret) = user.otp_secret else {
        bail!(ServerError::Internal("OTP secret missing".into()));
    };

    if !crate::totp::verify(secret, code)? {
        bail!(ServerError::UserError(errors::UserError::InvalidOtp));
    }

    return Ok(());
}

fn get_user(user_id: Uuid) -> ServerResult<UserModel> {
    let Some(user) = UserModel::find_by_id(user_id)? else {
        bail!(ServerError::Unauthenticated);
    };
    return Ok(user);
}

/// Decode an access token string into a session UUID without touching the database.
/// Used for logging/tracing where we only need the session ID, not full authentication.
pub fn decode_session_id(access_token: &str) -> Option<Uuid> {
    JWTToken::parse(access_token)
        .ok()
        .and_then(|jwt| jwt.decode().ok())
        .and_then(|payload| payload.subject_as_uuid().ok())
}

/// Extract a bearer token from HTTP headers (Authorization or WebSocket protocol).
pub fn extract_token_from_headers(headers: &http::HeaderMap) -> Option<String> {
    headers
        .get(http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer ").map(|t| t.to_string()))
        .or_else(|| {
            headers
                .get(http::header::SEC_WEBSOCKET_PROTOCOL)
                .and_then(|v| v.to_str().ok())
                .map(|t| t.to_string())
        })
}

/// Get all sessions for a user.
pub fn get_all_sessions(user_id: Uuid) -> ServerResult<Vec<SessionModel>> {
    return SessionModel::find_by_user(user_id);
}

/// Get all valid (non-closed) sessions for a user.
pub fn get_valid_sessions(user_id: Uuid) -> ServerResult<Vec<SessionModel>> {
    return SessionModel::find_valid_by_user(user_id);
}

/// Close all sessions for a user.
pub fn close_all_sessions(user_id: Uuid) -> ServerResult<()> {
    SessionModel::close_all_for_user(user_id, SessionInvalidationReason::UserClosed)
}

/// Close a specific session, verifying ownership.
pub fn close_session(user_id: Uuid, session_id: Uuid) -> ServerResult<()> {
    let Some(session) = SessionModel::find_by_id(session_id)? else {
        bail!(ServerError::NonExistentId(session_id.to_string()));
    };

    if session.user_id != user_id {
        bail!(ServerError::Unauthorized);
    }

    session.close(SessionInvalidationReason::UserClosed)
}

/// Delete all sessions for a user.
pub fn delete_all_sessions(user_id: Uuid) -> ServerResult<()> {
    SessionModel::delete_all_for_user(user_id)
}

/// Delete a specific session, verifying ownership.
pub fn delete_session(user_id: Uuid, session_id: Uuid) -> ServerResult<()> {
    let Some(session) = SessionModel::find_by_id(session_id)? else {
        bail!(ServerError::NonExistentId(session_id.to_string()));
    };

    if session.user_id != user_id {
        bail!(ServerError::Unauthorized);
    }

    session.delete()
}

/// Get the current user by ID.
pub fn get_current_user(user_id: Uuid) -> ServerResult<UserModel> {
    return get_user(user_id);
}

/// Update the current user's name. Assigns a new name_id for the new name.
pub fn update_user(user_id: Uuid, name: String) -> ServerResult<UserModel> {
    let mut errors = vec![];

    if name.len() < 4 || name.len() > 32 {
        errors.push(FieldError {
            field: String::from("name"),
            reason: FieldErrorReason::InvalidRange { min: 4, max: 32 },
        });
        bail!(body_error!(errors));
    }

    let mut user = get_user(user_id)?;
    if user.name != name {
        user.name_id = UserModel::assign_name_id(&name)?;
        user.name = name;
    }
    return user.persist();
}

/// Delete the current user and all their sessions.
pub fn delete_user(user_id: Uuid) -> ServerResult<()> {
    let _user = get_user(user_id)?;
    SessionModel::delete_all_for_user(user_id)?;
    UserModel::delete(user_id)?;
    return Ok(());
}

/// Parse a `name#0000` tag into (name, name_id).
fn parse_tag(input: &str) -> Option<(&str, i16)> {
    let (name, id_str) = input.rsplit_once('#')?;
    let id = id_str.parse::<i16>().ok()?;
    if !(0..10000).contains(&id) || name.is_empty() {
        return None;
    }
    return Some((name, id));
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── parse_tag ───────────────────────────────────────────────────────────

    #[test]
    fn parse_tag_valid() {
        assert_eq!(parse_tag("alice#1234"), Some(("alice", 1234)));
        assert_eq!(parse_tag("bob#0"), Some(("bob", 0)));
        assert_eq!(parse_tag("charlie#9999"), Some(("charlie", 9999)));
        assert_eq!(parse_tag("user#0001"), Some(("user", 1)));
    }

    #[test]
    fn parse_tag_no_hash() {
        assert_eq!(parse_tag("alice"), None);
    }

    #[test]
    fn parse_tag_empty_name() {
        assert_eq!(parse_tag("#1234"), None);
    }

    #[test]
    fn parse_tag_non_numeric_id() {
        assert_eq!(parse_tag("alice#abc"), None);
    }

    #[test]
    fn parse_tag_negative_id() {
        assert_eq!(parse_tag("alice#-1"), None);
    }

    #[test]
    fn parse_tag_id_out_of_range() {
        assert_eq!(parse_tag("alice#10000"), None);
        assert_eq!(parse_tag("alice#99999"), None);
    }

    #[test]
    fn parse_tag_multiple_hashes_uses_last() {
        // rsplit_once means "name#with#hash" → ("name#with", "hash")
        assert_eq!(parse_tag("name#with#1234"), Some(("name#with", 1234)));
    }

    #[test]
    fn parse_tag_empty_string() {
        assert_eq!(parse_tag(""), None);
    }

    #[test]
    fn parse_tag_only_hash() {
        assert_eq!(parse_tag("#"), None);
    }

    #[test]
    fn parse_tag_hash_no_id() {
        assert_eq!(parse_tag("alice#"), None);
    }

    #[test]
    fn parse_tag_overflow() {
        assert_eq!(parse_tag("alice#99999999999999"), None);
    }

    // ── Property tests ──────────────────────────────────────────────────────

    mod prop {
        use proptest::prelude::*;

        use super::super::parse_tag;

        proptest! {
            #[test]
            fn parse_tag_roundtrips(name in "[a-z]{1,20}", id in 0i16..10000) {
                let tag = format!("{}#{}", name, id);
                let result = parse_tag(&tag);
                prop_assert_eq!(result, Some((name.as_str(), id)));
            }

            #[test]
            fn parse_tag_rejects_out_of_range_ids(name in "[a-z]{1,20}", id in 10000i16..=i16::MAX) {
                let tag = format!("{}#{}", name, id);
                prop_assert_eq!(parse_tag(&tag), None);
            }

            #[test]
            fn parse_tag_always_returns_none_without_hash(input in "[a-z0-9]{0,50}") {
                if !input.contains('#') {
                    prop_assert_eq!(parse_tag(&input), None);
                }
            }

            #[test]
            fn parse_tag_never_returns_empty_name(input in ".*") {
                if let Some((name, _)) = parse_tag(&input) {
                    prop_assert!(!name.is_empty());
                }
            }

            #[test]
            fn parse_tag_id_always_in_range(input in ".{1,30}#[0-9]{1,5}") {
                if let Some((_, id)) = parse_tag(&input) {
                    prop_assert!(id >= 0 && id < 10000);
                }
            }
        }
    }
}
