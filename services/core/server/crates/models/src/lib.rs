#![allow(clippy::needless_return)]

pub mod entities;
mod init;
pub mod models;
mod schema;

// pub use entities::*;
pub use init::{DbInitFlags, initialize_database};
pub use models::*;
