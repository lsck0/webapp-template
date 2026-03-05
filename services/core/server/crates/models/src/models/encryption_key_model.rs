#![allow(clippy::needless_return)]

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use errors::ServerResult;
use uuid::Uuid;

use crate::{init::get_db, schema};

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = schema::encryption_keys)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewEncryptionKeyModel {
    pub key: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, AsChangeset, Queryable, Selectable)]
#[diesel(table_name = schema::encryption_keys)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct EncryptionKeyModel {
    pub id: Uuid,

    pub key: String,
    pub expires_at: DateTime<Utc>,

    pub updated_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl EncryptionKeyModel {
    pub fn new(new_key: NewEncryptionKeyModel) -> ServerResult<Self> {
        let key = diesel::insert_into(schema::encryption_keys::table)
            .values(&new_key)
            .get_result::<EncryptionKeyModel>(&mut get_db()?)?;

        return Ok(key);
    }

    /// Find all keys that haven't expired yet.
    pub fn find_all_valid() -> ServerResult<Vec<Self>> {
        let keys = schema::encryption_keys::table
            .filter(schema::encryption_keys::expires_at.gt(Utc::now()))
            .order(schema::encryption_keys::created_at.desc())
            .load::<EncryptionKeyModel>(&mut get_db()?)?;

        return Ok(keys);
    }

    /// Check if a key with this value already exists.
    pub fn find_by_key(key_value: &str) -> ServerResult<Option<Self>> {
        let key = schema::encryption_keys::table
            .filter(schema::encryption_keys::key.eq(key_value))
            .first::<EncryptionKeyModel>(&mut get_db()?)
            .optional()?;

        return Ok(key);
    }
}
