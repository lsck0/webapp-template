use aliri_braid::braid;
use base64::prelude::*;
use rand::RngExt;

/// A strongly-typed invite token used to authorize new user registrations.
#[braid(serde)]
pub struct InviteToken;

impl InviteToken {
    /// Create a new invite token using cryptographically secure random bytes.
    pub fn create() -> Self {
        let bytes: [u8; 32] = rand::rng().random();
        let token = BASE64_URL_SAFE_NO_PAD.encode(bytes);
        return Self::new(token);
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use pretty_assertions::{assert_eq, assert_ne};

    use super::*;

    #[test]
    fn test_invite_token_create() {
        let mut tokens = vec![];

        for _ in 0..10 {
            tokens.push(InviteToken::create().take());
        }

        tokens.into_iter().reduce(|a, b| {
            assert_ne!(a, b);
            b
        });
    }

    #[test]
    fn test_invite_token_is_base64() {
        let token = InviteToken::create().take();
        let decoded = BASE64_URL_SAFE_NO_PAD.decode(&token);
        assert!(decoded.is_ok(), "token should be valid base64url, got: {}", token);
    }

    #[test]
    fn test_invite_token_not_empty() {
        let token = InviteToken::create().take();
        assert!(!token.is_empty(), "invite token should not be empty");
    }

    #[test]
    fn test_invite_token_as_str_and_take() {
        let token = InviteToken::create();
        let as_str_val = token.as_str().to_string();
        let take_val = token.take();
        assert_eq!(as_str_val, take_val);
    }

    #[test]
    fn test_invite_token_uniqueness_large_batch() {
        let tokens: HashSet<String> = (0..100).map(|_| InviteToken::create().take()).collect();
        assert_eq!(tokens.len(), 100, "all 100 tokens should be unique");
    }
}
