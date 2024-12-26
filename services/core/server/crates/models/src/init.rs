use bitflags::bitflags;
use config::ServerConfig;
use crypto::HashedPassword;
use diesel::{
    connection::SimpleConnection,
    prelude::*,
    r2d2::{ConnectionManager, Pool, PooledConnection},
};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use errors::{ServerError, ServerResult};

use crate::{
    models::user_model::{NewUserModel, UserModel},
    permissions::Permissions,
    role_model::{NewRoleModel, RoleModel},
};

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations/");

static mut DB_POOL: Option<Pool<ConnectionManager<PgConnection>>> = None;

bitflags! {
    pub struct DbInitFlags: u8 {
        const NONE = 0b0000;
        const NUKE = 0b0001;
    }
}

/// Get a database connection from the database pool.
pub fn get_db() -> ServerResult<PooledConnection<ConnectionManager<PgConnection>>> {
    unsafe {
        let Some(ref pool) = DB_POOL else {
            panic!("Database pool not initialized.");
        };

        return pool.get().map_err(|_| ServerError::DBPoolExhausted);
    };
}

/// Initialize the database, database pool and default content.
/// If this is called in a test, it has to be annotated with #\[serial_test::serial\]!
pub fn initialize_database(flags: DbInitFlags) {
    let config = ServerConfig::get();

    let manager = ConnectionManager::<PgConnection>::new(config.database_url);

    let pool = Pool::builder()
        .max_size(config.database_pool_size)
        .build(manager)
        .expect("Failed to create the database pool.");

    unsafe {
        DB_POOL = Some(pool);
    }

    let mut connection = get_db().expect("Failed to access the database.");

    if flags.contains(DbInitFlags::NUKE) {
        connection
            .batch_execute("DROP SCHEMA public CASCADE; CREATE SCHEMA public;")
            .expect("Failed to drop and recreate the public schema.");
    }

    connection
        .run_pending_migrations(MIGRATIONS)
        .expect("Failed to run pending migrations.");

    if config.enable_default_user
        && !UserModel::find_by_name(&config.default_user_name).is_ok_and(|user| user.is_some())
        && !RoleModel::find_by_name("Admin").is_ok_and(|role| role.is_some())
    {
        let mut admin_role = RoleModel::new(NewRoleModel {
            name: String::from("Admin"),
            priority: 100,
        })
        .expect("Failed to create the default admin role.");

        admin_role.permissions = Permissions::all().into_iter().map(Some).collect();
        admin_role = admin_role.persist().expect("Failed to persist the default admin role.");

        let password_hash = HashedPassword::new(&config.default_user_password)
            .expect("Failed to hash the default admin password.")
            .consume();

        let admin_user = UserModel::new(NewUserModel {
            name: config.default_user_name,
            password_hash,
        })
        .expect("Failed to create the default admin user.");

        dbg!(&admin_user);

        admin_user
            .add_role(admin_role.id)
            .expect("Failed to add the default admin role to the default admin user.");
    }
}

#[cfg(test)]
mod tests {
    use serial_test::serial;

    use super::*;

    #[test]
    #[serial]
    fn test_initialize_database() {
        initialize_database(DbInitFlags::NUKE);

        let config = ServerConfig::get();

        UserModel::find_by_name(&config.default_user_name)
            .expect("Failed to access the database.")
            .expect("Default test user not found.");
    }

    #[test]
    #[serial]
    fn test_initialize_database_again() {
        initialize_database(DbInitFlags::NUKE);

        let config = ServerConfig::get();

        UserModel::find_by_name(&config.default_user_name)
            .expect("Failed to access the database.")
            .expect("Default test user not found.");
    }
}
