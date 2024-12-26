#![allow(clippy::needless_return)]

use std::sync::LazyLock;

use dotenvy::dotenv;
use macros::EnvVariables;

static CONFIG: LazyLock<ServerConfig> = LazyLock::new(|| {
    dotenv().ok();
    return ServerConfig::load_from_env();
});

/// Server configuration.
/// The fields in this struct correspond to environment variables.
#[derive(Debug, Clone, EnvVariables)]
pub struct ServerConfig {
    pub dev: bool,
    pub git_commit: String,

    pub database_pool_size: u32,
    pub database_url: String,

    pub enable_default_user: bool,
    pub default_user_name: String,
    pub default_user_password: String,

    pub jwt_secret: String,
    /// in minutes
    pub jwt_access_token_lifetime: u64,
    /// in days
    pub jwt_session_token_lifetime: u64,

    pub max_open_sessions: usize,
    /// per [login_attempt_timespan] minutes
    pub max_login_attempts: i32,
    /// in minutes
    pub login_attempt_timespan: i64,
    /// in minutes
    pub login_restriction_time: i64,
}

impl ServerConfig {
    pub fn get() -> Self {
        return CONFIG.clone();
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_config() {
        let config = ServerConfig::get();

        assert_eq!(config.dev, true);
    }
}
