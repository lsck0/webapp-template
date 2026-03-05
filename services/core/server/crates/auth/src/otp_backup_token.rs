use aliri_braid::braid;
use rand::RngExt;

/// A strongly-typed OTP backup/recovery code (7-digit numeric).
#[braid(serde)]
pub struct OTPBackupToken;

impl OTPBackupToken {
    /// Create a new OTP backup token.
    pub fn create() -> Self {
        let mut token = String::new();
        let mut rng = rand::rng();

        for _ in 0..7 {
            let digit = rng.random_range(0..10u8);
            token.push_str(&digit.to_string());
        }

        return Self::new(token);
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_otp_backup_token_create() {
        let token = OTPBackupToken::create().take();
        assert_eq!(token.len(), 7);
    }

    #[test]
    fn test_otp_backup_token_is_numeric() {
        let token = OTPBackupToken::create().take();
        assert!(
            token.chars().all(|c| c.is_ascii_digit()),
            "OTP backup token should contain only digits, got: {}",
            token
        );
    }

    #[test]
    fn test_otp_backup_token_as_str_and_take() {
        let token = OTPBackupToken::create();
        let as_str_val = token.as_str().to_string();
        let take_val = token.take();
        assert_eq!(as_str_val, take_val);
    }

    #[test]
    fn test_otp_backup_token_batch_all_numeric() {
        for _ in 0..50 {
            let token = OTPBackupToken::create().take();
            assert_eq!(token.len(), 7);
            assert!(token.chars().all(|c| c.is_ascii_digit()));
        }
    }

    #[test]
    fn test_otp_backup_token_uniqueness() {
        let tokens: HashSet<String> = (0..50).map(|_| OTPBackupToken::create().take()).collect();
        // With 10^7 possible values and 50 samples, collisions should be extremely rare
        assert!(tokens.len() >= 49, "most tokens should be unique, got {} unique out of 50", tokens.len());
    }
}
