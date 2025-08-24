use chrono::{DateTime, Utc};
use diesel::{deserialize::FromSqlRow, expression::AsExpression, prelude::*};
use errors::ServerResult;
use ipnetwork::IpNetwork;
use macros::PgText;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};
use ts_rs::TS;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    init::get_db,
    schema::{self, sessions},
};

#[derive(
    AsExpression,
    Clone,
    Copy,
    Debug,
    Deserialize,
    Display,
    EnumString,
    Eq,
    FromSqlRow,
    PartialEq,
    PgText,
    Serialize,
    TS,
    ToSchema,
)]
#[diesel(sql_type = diesel::sql_types::Text)]
#[ts(export)]
pub enum SessionInvalidationReason {
    Abandoned,
    AccessTokenLeak,
    Expired,
    SessionTokenLeak,
    UserClosed,
    UserLogout,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = schema::sessions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewSessionModel {
    pub user_id: Uuid,
    pub session_token: String,
    pub access_token: String,
    pub user_agent: String,
    pub ip_address: IpNetwork,
}

#[derive(Debug, Clone, AsChangeset, Queryable, Selectable)]
#[diesel(table_name = schema::sessions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct SessionModel {
    pub id: Uuid,

    pub user_id: Uuid,
    pub session_token: String,
    pub access_token: String,

    pub user_agent: String,
    pub ip_address: IpNetwork,
    pub last_used: DateTime<Utc>,

    pub valid: bool,
    pub invalidated_at: Option<DateTime<Utc>>,
    pub invalidated_reason: Option<SessionInvalidationReason>,

    pub updated_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl SessionModel {
    pub fn new(new_session: NewSessionModel) -> ServerResult<Self> {
        let session = diesel::insert_into(sessions::table)
            .values(&new_session)
            .get_result::<Self>(&mut get_db()?)?;

        return Ok(session);
    }

    pub fn find_by_id(id: Uuid) -> ServerResult<Option<Self>> {
        let session = sessions::table.find(id).first::<Self>(&mut get_db()?).optional()?;

        return Ok(session);
    }

    pub fn find_by_user(user_id: Uuid) -> ServerResult<Vec<Self>> {
        let sessions = sessions::table
            .filter(sessions::user_id.eq(user_id))
            .load::<Self>(&mut get_db()?)?;

        return Ok(sessions);
    }

    pub fn find_valid_by_user(user_id: Uuid) -> ServerResult<Vec<Self>> {
        let sessions = sessions::table
            .filter(sessions::user_id.eq(user_id))
            .filter(sessions::valid.eq(true))
            .load::<Self>(&mut get_db()?)?;

        return Ok(sessions);
    }

    pub fn persist(self) -> ServerResult<Self> {
        let post = diesel::update(sessions::table.filter(sessions::id.eq(self.id)))
            .set(self)
            .get_result::<Self>(&mut get_db()?)?;

        return Ok(post);
    }

    pub fn close(mut self, reason: SessionInvalidationReason) -> ServerResult<()> {
        self.valid = false;
        self.invalidated_at = Some(Utc::now());
        self.invalidated_reason = Some(reason);

        self.persist()?;

        return Ok(());
    }

    pub fn close_all_for_user(user_id: Uuid, reason: SessionInvalidationReason) -> ServerResult<()> {
        let sessions = sessions::table
            .filter(sessions::user_id.eq(user_id))
            .load::<Self>(&mut get_db()?)?;

        for session in sessions {
            session.close(reason)?;
        }

        return Ok(());
    }

    pub fn delete(self) -> ServerResult<()> {
        diesel::delete(sessions::table.filter(sessions::id.eq(self.id))).execute(&mut get_db()?)?;

        return Ok(());
    }

    pub fn delete_all_for_user(user_id: Uuid) -> ServerResult<()> {
        diesel::delete(sessions::table.filter(sessions::user_id.eq(user_id))).execute(&mut get_db()?)?;

        return Ok(());
    }
}
