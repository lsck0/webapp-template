#![allow(clippy::needless_return)]

use std::sync::Mutex;

use chrono::{Duration, Utc};
use config::ServerConfig;
use models::models::encryption_key_model::{EncryptionKeyModel, NewEncryptionKeyModel};
use rand::RngExt;

static KEY_RING: Mutex<Option<KeyRing>> = Mutex::new(None);

/// Holds the current signing key and all previous keys still valid for verification.
/// Initialized once after the database is ready.
#[derive(Debug, Clone)]
pub struct KeyRing {
    /// The active key used to sign new tokens.
    pub current: String,
    /// Previous keys that can still verify existing tokens (ordered newest-first).
    pub previous: Vec<String>,
}

impl KeyRing {
    /// Initialize the key ring from the database. Must be called once after DB initialization.
    ///
    /// - If no valid keys exist, generates a new one and inserts it.
    /// - The newest key becomes the current signing key.
    /// - All other non-expired keys are kept as previous keys for verification.
    pub fn initialize() {
        let config = ServerConfig::get();
        let mut db_keys = EncryptionKeyModel::find_all_valid().expect("Failed to load encryption keys from database.");

        // No keys yet — generate and insert one
        if db_keys.is_empty() {
            let expiry = Utc::now() + Duration::days(config.jwt_session_token_lifetime as i64 * 2);

            let new_key = EncryptionKeyModel::new(NewEncryptionKeyModel {
                key: Self::generate_secret(),
                expires_at: expiry,
            })
            .expect("Failed to create initial encryption key.");

            db_keys.push(new_key);
        }

        // Keys are ordered newest-first from the query
        let current = db_keys.remove(0).key;
        let previous: Vec<String> = db_keys.into_iter().map(|k| k.key).collect();

        let mut guard = KEY_RING.lock().unwrap();
        if guard.is_some() {
            panic!("KeyRing already initialized.");
        }
        *guard = Some(KeyRing { current, previous });
    }

    /// Generate a cryptographically random 64-character hex secret.
    fn generate_secret() -> String {
        let bytes: [u8; 32] = rand::rng().random();
        return bytes.iter().map(|b| format!("{b:02X}")).collect();
    }

    /// Initialize the key ring directly without DB access (for testing).
    #[cfg(test)]
    pub fn initialize_for_test(current: String, previous: Vec<String>) {
        let mut guard = KEY_RING.lock().unwrap();
        *guard = Some(KeyRing { current, previous });
    }

    /// Get the key ring. Panics if not initialized.
    pub fn get() -> KeyRing {
        let guard = KEY_RING.lock().unwrap();
        guard
            .clone()
            .expect("KeyRing not initialized. Call KeyRing::initialize() after DB init.")
    }

    /// Close/reset the key ring.
    pub fn close() {
        let mut guard = KEY_RING.lock().unwrap();
        *guard = None;
    }
}
