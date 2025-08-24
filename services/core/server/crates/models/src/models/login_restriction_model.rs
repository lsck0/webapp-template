use chrono::{DateTime, Utc};
use diesel::prelude::*;
use errors::ServerResult;
use uuid::Uuid;

use crate::{init::get_db, schema};

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = schema::login_restrictions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewLoginRestrictionModel {
    pub user_id: Uuid,
    pub restricted_until: DateTime<Utc>,
}

#[derive(Debug, Clone, AsChangeset, Queryable, Selectable)]
#[diesel(table_name = schema::login_restrictions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct LoginRestrictionModel {
    pub id: Uuid,

    pub user_id: Uuid,
    pub restricted_until: DateTime<Utc>,

    pub updated_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl LoginRestrictionModel {
    pub fn new(new_login_restriction: NewLoginRestrictionModel) -> ServerResult<Self> {
        let login_restriction = diesel::insert_into(schema::login_restrictions::table)
            .values(&new_login_restriction)
            .get_result::<LoginRestrictionModel>(&mut get_db()?)?;

        return Ok(login_restriction);
    }

    pub fn find_by_user_id(user_id: Uuid) -> ServerResult<Option<Self>> {
        let login_restriction = schema::login_restrictions::table
            .filter(schema::login_restrictions::user_id.eq(user_id))
            .first::<LoginRestrictionModel>(&mut get_db()?)
            .optional()?;

        return Ok(login_restriction);
    }
}
