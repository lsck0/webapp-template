use bitflags::bitflags;
use config::ServerConfig;
use diesel::{
    connection::SimpleConnection,
    prelude::*,
    r2d2::{ConnectionManager, Pool, PooledConnection},
};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use errors::{ServerError, ServerResult};

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

/// Close the database pool.
pub fn close_db() {
    unsafe {
        DB_POOL = None;
    }
}

/// Initialize the database pool and run pending migrations.
#[allow(clippy::redundant_closure_call)]
pub fn initialize_database(flags: DbInitFlags) {
    close_db();

    let config = ServerConfig::get();

    let manager = ConnectionManager::<PgConnection>::new(config.database_url);

    let pool = Pool::builder()
        .max_size(config.database_pool_size)
        .build(manager)
        .expect("Failed to create the database pool.");

    unsafe {
        DB_POOL = Some(pool);
    }

    let mut connection = get_db().expect("Failed to access the database while setting it up.");

    if flags.contains(DbInitFlags::NUKE) {
        connection
            .batch_execute("DROP SCHEMA public CASCADE; CREATE SCHEMA public;")
            .expect("Failed to drop and recreate the public schema.");
    }

    connection
        .run_pending_migrations(MIGRATIONS)
        .expect("Failed to run pending migrations.");
}

#[cfg(test)]
mod tests {
    use serial_test::serial;

    use super::*;

    #[test]
    #[serial]
    fn test_initialize_database() {
        initialize_database(DbInitFlags::NUKE);
        close_db();
    }

    #[test]
    #[serial]
    fn test_initialize_database_again() {
        initialize_database(DbInitFlags::NUKE);
        close_db();
    }
}
