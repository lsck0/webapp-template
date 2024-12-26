#![allow(clippy::needless_return)]

pub mod entities;
mod init;
pub mod models;
mod schema;

// pub use entities::*;
pub use init::{initialize_database, DbInitFlags};
pub use models::*;
