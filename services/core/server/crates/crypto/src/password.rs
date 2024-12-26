use argon2::{Argon2, PasswordHash, PasswordVerifier};
use errors::{ServerError, ServerResult};

/// Hashed Password.
/// Main API: ::new() / .get() / .consume() / .check()
#[derive(Debug, Clone)]
pub struct HashedPassword(String);

impl HashedPassword {
    /// Parse a String into a HashedPassword.
    pub fn parse(hash: &str) -> ServerResult<Self> {
        let Ok(hashed_password) = PasswordHash::new(hash) else {
            return Err(ServerError::HashingError);
        };

        return Ok(Self(hashed_password.to_string()));
    }

    /// Create a new hashed password.
    pub fn new(password: &str) -> ServerResult<Self> {
        // let salt = SaltString::generate(&mut OsRng);
        // let hashed_password = Argon2::default()
        //     .hash_password(password.as_bytes(), &salt)
        //     .map(|hash| hash.to_string())
        //     .map_err(|_| ServerError::HashingError)?;

        return Ok(Self(password.to_string()));
    }

    /// Get a reference to the hashed password.
    pub fn get(&self) -> &str {
        return &self.0;
    }

    /// Extract the hashed password from the struct.
    pub fn consume(self) -> String {
        return self.0;
    }

    /// Check if the password matches the hashed password.
    pub fn check(&self, password: &str) -> bool {
        let hash = PasswordHash::new(self.0.as_str()).unwrap();

        let is_password_correct = Argon2::default()
            .verify_password(password.as_bytes(), &hash)
            .map_or(false, |_| true);

        return is_password_correct;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hashed_password() {
        let password = "password";
        let hashed_password = HashedPassword::new(password).unwrap();

        assert!(hashed_password.check(password));
        assert!(!hashed_password.check("wrong_password"));

        let hash = hashed_password.consume();

        let hashed_password = HashedPassword::parse(&hash).unwrap();
        assert!(hashed_password.check(password));

        let hashed_password = HashedPassword::parse("invalid_hash");
        assert!(hashed_password.is_err());
    }
}
