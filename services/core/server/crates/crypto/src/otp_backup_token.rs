use rand::random;
use serde::{Deserialize, Serialize};

/// OTP Backup Token.
/// Main API: ::create() / .get() / .consume()
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OTPBackupToken(String);

impl OTPBackupToken {
    /// Create a new OTP backup token.
    pub fn create() -> Self {
        let mut token = String::new();

        for _ in 0..7 {
            let digit = random::<u8>() % 10;
            token.push_str(&digit.to_string());
        }

        return Self(token);
    }

    /// Get a reference to the token.
    pub fn get(&self) -> &str {
        return &self.0;
    }

    /// Extract the token from the struct.
    pub fn consume(self) -> String {
        return self.0;
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_top_backup_token_create() {
        let token = OTPBackupToken::create().consume();

        assert_eq!(token.len(), 7);
    }
}
