//! Crypto crate for password hashing, JWT, OTP, and invite tokens.

#![allow(clippy::needless_return)]
#![feature(duration_constructors, random)]

mod invite_token;
mod jwt_token;
mod otp_backup_token;
mod password;

pub use invite_token::InviteToken;
pub use jwt_token::JWTToken;
pub use otp_backup_token::OTPBackupToken;
pub use password::HashedPassword;
