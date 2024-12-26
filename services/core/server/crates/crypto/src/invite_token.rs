use std::hash::{DefaultHasher, Hash, Hasher};

use base64::prelude::*;
use chrono::Utc;
use rand::random;
use serde::{Deserialize, Serialize};

/// Invite Token.
/// Main API: ::create() / .get() / .consume()
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InviteToken(String);

impl InviteToken {
    /// Create a new invite token.
    pub fn create() -> Self {
        let now = Utc::now().timestamp_millis();
        let nonce = random::<u64>();

        let raw_token = format!("invite-{}-{}", nonce, now);

        let mut hasher = DefaultHasher::new();
        raw_token.hash(&mut hasher);
        let hashed_token = hasher.finish();

        let token = BASE64_STANDARD_NO_PAD.encode(hashed_token.to_string());

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
    use pretty_assertions::assert_ne;

    use super::*;

    #[test]
    fn test_invite_token_create() {
        let mut tokens = vec![];

        for _ in 0..10 {
            tokens.push(InviteToken::create().consume());
        }

        tokens.into_iter().reduce(|a, b| {
            assert_ne!(a, b);
            b
        });
    }
}
