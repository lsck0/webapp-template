use std::time::Duration;

use aliri_braid::braid;
use chrono::Utc;
use config::ServerConfig;
use errors::{ServerError, ServerResult};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, decode_header, encode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::keyring::KeyRing;

/// A strongly-typed JWT token string.
#[braid(serde)]
pub struct JWTToken;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPayload {
    pub sub: String,
    pub iat: usize,
    pub exp: usize,
}

impl JWTToken {
    /// Parse a String into a JWTToken.
    pub fn parse(string: &str) -> ServerResult<Self> {
        if decode_header(string).is_ok() {
            return Ok(JWTToken::new(string.to_string()));
        }

        return Err(ServerError::InvalidToken);
    }

    /// Create a new session token.
    pub fn new_session_token(session_id: &Uuid) -> Self {
        let days = ServerConfig::get().jwt_session_token_lifetime;
        let lifetime = Duration::from_days(days);

        return JWTToken::new(Self::generate_token(session_id.to_string(), lifetime));
    }

    /// Create a new access token.
    pub fn new_access_token(session_id: &Uuid) -> Self {
        let minutes = ServerConfig::get().jwt_access_token_lifetime;
        let lifetime = Duration::from_mins(minutes);

        return JWTToken::new(Self::generate_token(session_id.to_string(), lifetime));
    }

    /// Decode the token into a TokenPayload.
    /// Tries the current signing key first, then falls back to previous (rotated) keys.
    pub fn decode(&self) -> ServerResult<TokenPayload> {
        let keyring = KeyRing::get();

        // Try the current key first
        match Self::decode_with_key(self.as_str(), &keyring.current) {
            Ok(payload) => return Ok(payload),
            Err(ServerError::InvalidToken) => {
                // InvalidToken from signature mismatch — try previous keys
            }
            Err(other) => return Err(other),
        }

        // Try each previous key
        for prev_key in &keyring.previous {
            match Self::decode_with_key(self.as_str(), prev_key) {
                Ok(payload) => return Ok(payload),
                Err(ServerError::InvalidToken) => continue,
                Err(other) => return Err(other),
            }
        }

        return Err(ServerError::InvalidToken);
    }

    /// Decode a token using a specific secret key.
    fn decode_with_key(token: &str, secret: &str) -> ServerResult<TokenPayload> {
        let decoded_token = decode::<TokenPayload>(
            token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &Validation::default(),
        );

        // BUG: manually check expiration, the library failed to do so for some reason.
        let decoded_token = decoded_token.and_then(|token| {
            if token.claims.exp < Utc::now().timestamp() as usize {
                return Err(jsonwebtoken::errors::Error::from(
                    jsonwebtoken::errors::ErrorKind::ExpiredSignature,
                ));
            }

            return Ok(token);
        });

        use jsonwebtoken::errors::ErrorKind;
        return match decoded_token {
            Ok(token) => Ok(token.claims),
            Err(err) if err.kind() == &ErrorKind::InvalidToken => Err(ServerError::InvalidToken),
            Err(err) if err.kind() == &ErrorKind::InvalidSignature => Err(ServerError::InvalidToken),
            Err(err) if err.kind() == &ErrorKind::ExpiredSignature => Err(ServerError::ExpiredToken),
            Err(err) => Err(ServerError::from(err)),
        };
    }

    /// Generate a new token. Always signs with the current key from the key ring.
    fn generate_token(subject: String, lifetime: Duration) -> String {
        let now = Utc::now();
        let payload = TokenPayload {
            sub: subject,
            iat: now.timestamp() as usize,
            exp: (now + lifetime).timestamp() as usize,
        };

        let mut token = encode(
            &Header::default(),
            &payload,
            &EncodingKey::from_secret(KeyRing::get().current.as_ref()),
        )
        .expect("Failed to encode JWT token.");

        // BUG: There seems to be an issue with subjects less than 6 characters long.
        // We exclusively use UUIDs as subjects, so this is not an issue here.

        // The length of a base64 encoded string is always a multiple of 4.
        // If it is not a multiple of 4, then = characters are appended until it is.
        // https://stackoverflow.com/questions/2925729/invalid-length-for-a-base-64-char-array
        if token.len() % 4 != 0 {
            let padding = 4 - (token.len() % 4);
            token.push_str(&"=".repeat(padding));
        }

        return token;
    }
}

impl TokenPayload {
    /// Get the subject as a UUID.
    pub fn subject_as_uuid(&self) -> ServerResult<Uuid> {
        return Uuid::parse_str(&self.sub).map_err(|_| ServerError::InvalidToken);
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Once;

    use pretty_assertions::assert_eq;
    use proptest::prelude::*;
    use rstest::rstest;

    use super::*;

    static INIT: Once = Once::new();

    fn init_keyring() {
        INIT.call_once(|| {
            KeyRing::initialize_for_test(
                String::from("test-secret-key-for-jwt-unit-tests"),
                vec![],
            );
        });
    }

    #[test]
    fn test_jwt_token() {
        init_keyring();
        let session_id = Uuid::new_v4();
        let token = JWTToken::new_session_token(&session_id);

        let token_session_id = token.decode().unwrap().subject_as_uuid().unwrap();

        assert_eq!(session_id, token_session_id);
    }

    #[test]
    fn test_jwt_token_expired() {
        init_keyring();
        let token = JWTToken::generate_token(String::from("xxxxxx"), Duration::from_secs(0));
        std::thread::sleep(Duration::from_secs(1));

        let decode_result = JWTToken::parse(&token).unwrap().decode();

        assert!(
            matches!(decode_result, Err(ServerError::ExpiredToken)),
            "Unexpected result: {:?}",
            decode_result
        );
    }

    #[test]
    fn test_jwt_token_invalid() {
        init_keyring();
        let token = JWTToken::generate_token(String::from("xxxxxx"), Duration::from_days(1));
        let invalid_token = token.chars().rev().collect::<String>();

        let parse_result = JWTToken::parse(&invalid_token);

        assert!(
            matches!(parse_result, Err(ServerError::InvalidToken)),
            "Unexpected result: {:?}",
            parse_result
        );
    }

    #[test]
    fn test_access_token_roundtrip() {
        init_keyring();
        let session_id = Uuid::new_v4();
        let token = JWTToken::new_access_token(&session_id);

        let payload = token.decode().unwrap();
        let decoded_id = payload.subject_as_uuid().unwrap();

        assert_eq!(session_id, decoded_id);
    }

    #[test]
    fn test_token_as_str_and_take() {
        init_keyring();
        let session_id = Uuid::new_v4();
        let token = JWTToken::new_session_token(&session_id);

        let token_str = token.as_str().to_string();
        let taken = token.take();

        assert_eq!(token_str, taken);
    }

    #[test]
    fn test_parse_empty_string() {
        let result = JWTToken::parse("");
        assert!(matches!(result, Err(ServerError::InvalidToken)));
    }

    #[rstest]
    #[case("not-a-jwt")]
    #[case("abc.def.ghi")]
    #[case("eyJ.invalid.token")]
    fn test_parse_garbage_strings(#[case] input: &str) {
        let result = JWTToken::parse(input);
        assert!(matches!(result, Err(ServerError::InvalidToken)));
    }

    #[test]
    fn test_payload_iat_before_exp() {
        init_keyring();
        let session_id = Uuid::new_v4();
        let token = JWTToken::new_session_token(&session_id);
        let payload = token.decode().unwrap();

        assert!(payload.iat < payload.exp, "iat ({}) should be before exp ({})", payload.iat, payload.exp);
    }

    #[test]
    fn test_subject_as_uuid_invalid() {
        init_keyring();
        let session_id = Uuid::new_v4();
        let token = JWTToken::new_session_token(&session_id);
        let mut payload = token.decode().unwrap();

        // Corrupt the subject so it's no longer a valid UUID
        payload.sub = String::from("not-a-uuid");
        let result = payload.subject_as_uuid();
        assert!(matches!(result, Err(ServerError::InvalidToken)));
    }

    #[test]
    fn test_session_and_access_tokens_differ() {
        init_keyring();
        let session_id = Uuid::new_v4();
        let session_token = JWTToken::new_session_token(&session_id).take();
        let access_token = JWTToken::new_access_token(&session_id).take();

        assert_ne!(session_token, access_token, "session and access tokens should differ");
    }

    /// Verify that a token signed with a previous key can still be decoded.
    #[test]
    fn test_decode_with_previous_key() {
        init_keyring();

        let old_secret = "old-secret-key-for-testing-12345";
        let new_secret = "new-secret-key-for-testing-67890";

        // Sign a token with the old key
        let session_id = Uuid::new_v4();
        let payload = TokenPayload {
            sub: session_id.to_string(),
            iat: Utc::now().timestamp() as usize,
            exp: (Utc::now() + Duration::from_days(1)).timestamp() as usize,
        };

        let old_token = encode(
            &Header::default(),
            &payload,
            &EncodingKey::from_secret(old_secret.as_bytes()),
        )
        .unwrap();

        // Decode should succeed with old_secret as a previous key
        let result = JWTToken::decode_with_key(&old_token, old_secret);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().subject_as_uuid().unwrap(), session_id);

        // Decode should fail with new_secret (wrong key)
        let result = JWTToken::decode_with_key(&old_token, new_secret);
        assert!(matches!(result, Err(ServerError::InvalidToken)));
    }

    proptest! {
        #[test]
        fn test_jwt_roundtrip_any_uuid(
            a in any::<u128>()
        ) {
            init_keyring();
            let session_id = Uuid::from_u128(a);
            let token = JWTToken::new_session_token(&session_id);
            let decoded = token.decode().unwrap().subject_as_uuid().unwrap();
            prop_assert_eq!(session_id, decoded);
        }
    }
}
