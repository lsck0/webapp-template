use chrono::{DateTime, Utc};
use diesel::prelude::*;
use errors::ServerResult;
use uuid::Uuid;

use crate::{init::get_db, permissions::Permissions, schema};

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = schema::roles)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewRoleModel {
    pub name: String,
    pub priority: i32,
}

#[derive(Debug, Clone, Queryable, Selectable, AsChangeset)]
#[diesel(table_name = schema::roles)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct RoleModel {
    pub id: Uuid,

    pub name: String,
    pub priority: i32,
    pub permissions: Vec<Option<Permissions>>,
    pub permissions_forbidden: Vec<Option<Permissions>>,

    pub updated_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = schema::user_roles)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewUserRoleModel {
    pub user_id: Uuid,
    pub role_id: Uuid,
}

#[derive(Debug, Clone, Queryable, Selectable, AsChangeset)]
#[diesel(table_name = schema::user_roles)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UserRoleModel {
    pub user_id: Uuid,
    pub role_id: Uuid,

    pub updated_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl RoleModel {
    pub fn new(new_role: NewRoleModel) -> ServerResult<Self> {
        let role = diesel::insert_into(schema::roles::table)
            .values(&new_role)
            .get_result::<Self>(&mut get_db()?)?;

        return Ok(role);
    }

    pub fn find_by_id(id: Uuid) -> ServerResult<Option<Self>> {
        let role = schema::roles::table.find(id).first::<Self>(&mut get_db()?).optional()?;

        return Ok(role);
    }

    pub fn find_by_name(name: &str) -> ServerResult<Option<Self>> {
        let role = schema::roles::table
            .filter(schema::roles::name.eq(name))
            .first::<Self>(&mut get_db()?)
            .optional()?;

        return Ok(role);
    }

    pub fn get_all() -> ServerResult<Vec<Self>> {
        let roles = schema::roles::table.load::<Self>(&mut get_db()?)?;

        return Ok(roles);
    }

    pub fn persist(self) -> ServerResult<Self> {
        let role = diesel::update(schema::roles::table)
            .filter(schema::roles::id.eq(self.id))
            .set(self)
            .get_result::<Self>(&mut get_db()?)?;

        return Ok(role);
    }

    pub fn delete(self) -> ServerResult<()> {
        diesel::delete(schema::roles::table.filter(schema::roles::id.eq(self.id))).execute(&mut get_db()?)?;

        return Ok(());
    }
}
