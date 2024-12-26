use std::time::Duration;

use chrono::Utc;
use config::ServerConfig;
use errors::{ServerError, ServerResult};
use jsonwebtoken::{decode, decode_header, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// JWT Token.
/// Main API: ::parse() / ::new_session_token() / ::new_access_token() / .decode() / .get() /
/// .consume()
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JWTToken(String);

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
            return Ok(JWTToken(string.to_string()));
        }

        return Err(ServerError::InvalidToken);
    }

    /// Create a new session token.
    pub fn new_session_token(session_id: &Uuid) -> Self {
        let days = ServerConfig::get().jwt_session_token_lifetime;
        let lifetime = Duration::from_days(days);

        return JWTToken(Self::generate_token(session_id.to_string(), lifetime));
    }

    /// Create a new access token.
    pub fn new_access_token(session_id: &Uuid) -> Self {
        let minutes = ServerConfig::get().jwt_access_token_lifetime;
        let lifetime = Duration::from_mins(minutes);

        return JWTToken(Self::generate_token(session_id.to_string(), lifetime));
    }

    /// Decode the token into a TokenPayload.
    pub fn decode(&self) -> ServerResult<TokenPayload> {
        let decoded_token = decode::<TokenPayload>(
            &self.0,
            &DecodingKey::from_secret(ServerConfig::get().jwt_secret.as_bytes()),
            &Validation::default(),
        );

        // BUG: manually check expiration, the library failed to do so for some reason.
        let decoded_token = decoded_token.and_then(|token| {
            if token.claims.exp <= Utc::now().timestamp() as usize {
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
            Err(err) if err.kind() == &ErrorKind::ExpiredSignature => Err(ServerError::ExpiredToken),
            Err(err) => Err(ServerError::from(err)),
        };
    }

    /// Get a reference to the token.
    pub fn get(&self) -> &str {
        return &self.0;
    }

    /// Extract the token from the struct.
    pub fn consume(self) -> String {
        return self.0;
    }

    /// Generate a new token.
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
            &EncodingKey::from_secret(ServerConfig::get().jwt_secret.as_ref()),
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
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_jwt_token() {
        let session_id = Uuid::new_v4();
        let token = JWTToken::new_session_token(&session_id);

        let token_session_id = token.decode().unwrap().subject_as_uuid().unwrap();

        assert_eq!(session_id, token_session_id);
    }

    #[test]
    fn test_jwt_token_expired() {
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
        let token = JWTToken::generate_token(String::from("xxxxxx"), Duration::from_days(1));
        let invalid_token = token.chars().rev().collect::<String>();

        let parse_result = JWTToken::parse(&invalid_token);

        assert!(
            matches!(parse_result, Err(ServerError::InvalidToken)),
            "Unexpected result: {:?}",
            parse_result
        );
    }
}
