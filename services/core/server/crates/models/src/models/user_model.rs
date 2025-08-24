use chrono::{DateTime, Utc};
use diesel::prelude::*;
use errors::ServerResult;
use uuid::Uuid;

use crate::{
    init::get_db,
    permissions::Permissions,
    role_model::NewUserRoleModel,
    schema::{self, users},
};

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewUserModel {
    pub name: String,
    pub password_hash: String,
}

#[derive(Debug, Clone, AsChangeset, Queryable, Selectable)]
#[diesel(table_name = schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UserModel {
    pub id: Uuid,

    pub name: String,

    pub password_hash: String,

    pub otp_enabled: bool,
    pub otp_validated: bool,
    pub otp_secret: Option<String>,
    pub otp_url: Option<String>,
    pub otp_recovery_codes: Option<Vec<Option<String>>>,

    pub permissions: Vec<Option<Permissions>>,
    pub permissions_forbidden: Vec<Option<Permissions>>,

    pub updated_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl UserModel {
    pub fn new(new_user: NewUserModel) -> ServerResult<Self> {
        let user = diesel::insert_into(users::table)
            .values(&new_user)
            .get_result::<UserModel>(&mut get_db()?)?;

        return Ok(user);
    }

    pub fn get_all() -> ServerResult<Vec<Self>> {
        let users = users::table.load::<UserModel>(&mut get_db()?)?;

        return Ok(users);
    }

    pub fn find_by_id(id: Uuid) -> ServerResult<Option<Self>> {
        let user = users::table.find(id).first::<UserModel>(&mut get_db()?).optional()?;

        return Ok(user);
    }

    pub fn find_by_name(name: &str) -> ServerResult<Option<Self>> {
        let user = users::table
            .filter(users::name.eq(name))
            .first::<UserModel>(&mut get_db()?)
            .optional()?;

        return Ok(user);
    }

    pub fn persist(self) -> ServerResult<Self> {
        let user = diesel::update(users::table.find(self.id))
            .set(self)
            .get_result::<UserModel>(&mut get_db()?)?;

        return Ok(user);
    }

    pub fn add_role(self, role_id: Uuid) -> ServerResult<()> {
        let new_user_role = NewUserRoleModel {
            user_id: self.id,
            role_id,
        };

        diesel::insert_into(schema::user_roles::table)
            .values(&new_user_role)
            .execute(&mut get_db()?)?;

        return Ok(());
    }

    pub fn delete(id: Uuid) -> ServerResult<()> {
        diesel::delete(users::table.find(id)).execute(&mut get_db()?)?;

        return Ok(());
    }
}
