use chrono::{DateTime, Duration, Utc};
use config::ServerConfig;
use diesel::prelude::*;
use errors::ServerResult;
use uuid::Uuid;

use crate::{
    init::get_db,
    schema::{self, login_attempts},
};

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = schema::login_attempts)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewLoginAttemptModel {
    pub user_id: Uuid,
}

#[derive(Debug, Clone, AsChangeset, Queryable, Selectable)]
#[diesel(table_name = schema::login_attempts)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct LoginAttemptModel {
    pub id: Uuid,

    pub user_id: Uuid,
    pub failed_counter: i32,
    pub last_attempt: DateTime<Utc>,

    pub updated_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl LoginAttemptModel {
    pub fn new(user_id: Uuid) -> ServerResult<Self> {
        let new_attempt = NewLoginAttemptModel { user_id };

        let attempt = diesel::insert_into(login_attempts::table)
            .values(&new_attempt)
            .get_result::<LoginAttemptModel>(&mut get_db()?)?;

        return Ok(attempt);
    }

    pub fn find_past_attempt(user_id: &Uuid) -> ServerResult<Option<Self>> {
        let past_attempt = login_attempts::table
            .filter(login_attempts::user_id.eq(user_id))
            .filter(
                login_attempts::last_attempt
                    .ge(Utc::now() - Duration::minutes(ServerConfig::get().login_attempt_timespan)),
            )
            .first::<LoginAttemptModel>(&mut get_db()?)
            .optional()?;

        return Ok(past_attempt);
    }

    pub fn persist(self) -> ServerResult<Self> {
        let attempt = diesel::update(login_attempts::table)
            .filter(login_attempts::id.eq(self.id))
            .set(self)
            .get_result::<LoginAttemptModel>(&mut get_db()?)?;

        return Ok(attempt);
    }

    pub fn delete_all_for_user(user_id: &Uuid) -> ServerResult<()> {
        diesel::delete(login_attempts::table.filter(login_attempts::user_id.eq(user_id))).execute(&mut get_db()?)?;

        return Ok(());
    }
}
