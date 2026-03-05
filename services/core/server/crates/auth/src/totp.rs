#![allow(clippy::needless_return)]

use errors::{ServerError, ServerResult};
use totp_rs::{Algorithm, Secret, TOTP};

const ISSUER: &str = "WebApp";
const DIGITS: usize = 6;
const STEP: u64 = 30;
const SKEW: u8 = 1;

/// Generate a new TOTP secret and otpauth URL for a user.
/// Returns (base32_secret, otpauth_url).
pub fn generate_secret(account_name: &str) -> ServerResult<(String, String)> {
    let secret = Secret::generate_secret();
    let base32 = secret.to_encoded().to_string();
    let bytes = secret.to_bytes().map_err(|_| ServerError::Internal("Bad TOTP secret".into()))?;

    let totp = build_totp(bytes, account_name)?;
    let url = totp.get_url();

    return Ok((base32, url));
}

/// Generate a QR code PNG as a base64-encoded data URI from an otpauth URL.
pub fn generate_qr_data_uri(account_name: &str, secret_bytes: &[u8]) -> ServerResult<String> {
    let totp = build_totp(secret_bytes.to_vec(), account_name)?;
    let qr = totp.get_qr_base64().map_err(|_| ServerError::Internal("Failed to generate QR code".into()))?;
    return Ok(format!("data:image/png;base64,{qr}"));
}

/// Verify a TOTP code against a secret.
pub fn verify(secret_base32: &str, code: &str) -> ServerResult<bool> {
    let secret = Secret::Encoded(secret_base32.to_string())
        .to_bytes()
        .map_err(|_| ServerError::InvalidToken)?;

    let totp = build_totp(secret, "verify")?;
    return Ok(totp.check_current(code).unwrap_or(false));
}

fn build_totp(secret: Vec<u8>, account_name: &str) -> ServerResult<TOTP> {
    let totp = TOTP::new(Algorithm::SHA1, DIGITS, SKEW, STEP, secret, Some(ISSUER.to_string()), account_name.to_string())
        .map_err(|_| ServerError::Internal("Failed to create TOTP".into()))?;
    return Ok(totp);
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn generate_secret_returns_valid_base32_and_url() {
        let (secret, url) = generate_secret("testuser").unwrap();
        assert!(!secret.is_empty());
        assert!(url.starts_with("otpauth://totp/"));
        assert!(url.contains("WebApp"));
        assert!(url.contains("testuser"));
    }

    #[test]
    fn generate_qr_data_uri_returns_png_data_uri() {
        let secret = Secret::generate_secret();
        let bytes = secret.to_bytes().unwrap();
        let uri = generate_qr_data_uri("testuser", &bytes).unwrap();
        assert!(uri.starts_with("data:image/png;base64,"));
    }

    #[test]
    fn verify_accepts_current_code() {
        let secret = Secret::generate_secret();
        let base32 = secret.to_encoded().to_string();
        let bytes = secret.to_bytes().unwrap();
        let totp = build_totp(bytes, "test").unwrap();
        let code = totp.generate_current().unwrap();

        assert_eq!(verify(&base32, &code).unwrap(), true);
    }

    #[test]
    fn verify_rejects_wrong_code() {
        let (secret, _) = generate_secret("test").unwrap();
        assert_eq!(verify(&secret, "000000").unwrap(), false);
    }

    #[test]
    fn verify_rejects_invalid_secret() {
        assert!(verify("not-valid-base32!!!", "123456").is_err());
    }
}
