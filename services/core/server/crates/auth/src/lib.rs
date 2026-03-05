#![allow(clippy::needless_return)]
#![feature(duration_constructors, random)]

pub mod defense;
mod init;
mod invite_token;
mod jwt_token;
pub mod keyring;
mod otp_backup_token;
mod password;
mod service;
pub mod totp;

pub use init::initialize_auth;
pub use invite_token::InviteToken;
pub use jwt_token::{JWTToken, TokenPayload};
pub use otp_backup_token::OTPBackupToken;
pub use password::HashedPassword;
pub use service::{
    LoginRequest, OtpEnableResponse, PasswordChangeRequest, RefreshRequest, RegisterRequest,
    authenticate, change_password, close_all_sessions, close_session, create_invite,
    decode_session_id, delete_all_sessions, delete_session, delete_user,
    extract_token_from_headers, get_all_sessions, get_current_user, get_valid_sessions, login,
    logout, otp_disable, otp_enable, otp_regenerate_backup_codes, otp_validate, refresh, register,
    require_permission, resolve_permissions, update_user,
};
