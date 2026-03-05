use aliri_braid::braid;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier, password_hash::SaltString};
use errors::{ServerError, ServerResult};

/// A strongly-typed argon2 password hash.
#[braid]
pub struct HashedPassword;

impl HashedPassword {
    /// Parse a String into a HashedPassword, validating it's a valid argon2 hash.
    pub fn parse(hash: &str) -> ServerResult<Self> {
        let Ok(hashed_password) = PasswordHash::new(hash) else {
            return Err(ServerError::HashingError);
        };

        return Ok(Self::new(hashed_password.to_string()));
    }

    /// Create a new hashed password from plaintext.
    pub fn hash(password: &str) -> ServerResult<Self> {
        let raw: [u8; 16] = rand::random();
        let salt = SaltString::encode_b64(&raw).map_err(|_| ServerError::HashingError)?;
        let hashed_password = Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map(|hash| hash.to_string())
            .map_err(|_| ServerError::HashingError)?;

        return Ok(Self::new(hashed_password));
    }

    /// Check if the password matches the hashed password.
    pub fn check(&self, password: &str) -> bool {
        let hash = PasswordHash::new(self.as_str()).unwrap();

        let is_password_correct = Argon2::default()
            .verify_password(password.as_bytes(), &hash)
            .is_ok_and(|_| true);

        return is_password_correct;
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use proptest::prelude::*;
    use rstest::rstest;

    use super::*;

    #[test]
    fn test_hashed_password() {
        let password = "password";
        let hashed_password = HashedPassword::hash(password).unwrap();

        assert!(hashed_password.check(password));
        assert!(!hashed_password.check("wrong_password"));

        let hash = hashed_password.take();

        let hashed_password = HashedPassword::parse(&hash).unwrap();
        assert!(hashed_password.check(password));

        let hashed_password = HashedPassword::parse("invalid_hash");
        assert!(hashed_password.is_err());
    }

    #[test]
    fn test_hash_is_not_plaintext() {
        let password = "my_secret_password";
        let hashed = HashedPassword::hash(password).unwrap();
        assert_ne!(hashed.as_str(), password, "hash should not be the plaintext password");
    }

    #[test]
    fn test_different_passwords_produce_different_hashes() {
        let hash1 = HashedPassword::hash("password_one").unwrap().take();
        let hash2 = HashedPassword::hash("password_two").unwrap().take();
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_same_password_produces_different_hashes() {
        let hash1 = HashedPassword::hash("same_password").unwrap().take();
        let hash2 = HashedPassword::hash("same_password").unwrap().take();
        assert_ne!(hash1, hash2, "same password should produce different hashes due to salt");
    }

    #[test]
    fn test_hash_starts_with_argon2_prefix() {
        let hashed = HashedPassword::hash("test_password").unwrap();
        assert!(
            hashed.as_str().starts_with("$argon2"),
            "hash should start with $argon2, got: {}",
            hashed.as_str()
        );
    }

    #[test]
    fn test_as_str_and_take_return_same_value() {
        let hashed = HashedPassword::hash("test_password").unwrap();
        let as_str_val = hashed.as_str().to_string();
        let take_val = hashed.take();
        assert_eq!(as_str_val, take_val);
    }

    #[rstest]
    #[case("short")]
    #[case("a-longer-password-with-special-chars-!@#$%^&*()")]
    #[case("unicode-пароль-密码")]
    #[case("  spaces  ")]
    #[case("")]
    fn test_various_passwords(#[case] password: &str) {
        let hashed = HashedPassword::hash(password).unwrap();
        assert!(hashed.check(password), "password '{}' should verify", password);
    }

    #[rstest]
    #[case("correct", "incorrect")]
    #[case("password", "Password")]
    #[case("password", "password ")]
    #[case("password", " password")]
    fn test_wrong_passwords_fail(#[case] password: &str, #[case] wrong: &str) {
        let hashed = HashedPassword::hash(password).unwrap();
        assert!(!hashed.check(wrong), "password '{}' should not match hash of '{}'", wrong, password);
    }

    #[rstest]
    #[case("")]
    #[case("not-a-hash")]
    #[case("plaintext_password")]
    fn test_parse_invalid_hashes(#[case] input: &str) {
        let result = HashedPassword::parse(input);
        assert!(result.is_err(), "parsing '{}' should fail", input);
    }

    proptest! {
        #[test]
        fn test_hash_verify_roundtrip(password in "[a-zA-Z0-9!@#$%^&*]{1,32}") {
            let hashed = HashedPassword::hash(&password).unwrap();
            prop_assert!(hashed.check(&password));
        }
    }
}
