use chrono::{DateTime, Duration, Utc};
use diesel::prelude::*;
use errors::ServerResult;
use uuid::Uuid;

use crate::{
    init::get_db,
    schema::{self, invites},
};

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = schema::invites)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewInviteModel {
    pub created_by: Uuid,
    pub token: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, AsChangeset, Queryable, Selectable)]
#[diesel(table_name = schema::invites)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct InviteModel {
    pub id: Uuid,

    pub created_by: Uuid,
    pub token: String,
    pub expires_at: DateTime<Utc>,

    pub used: bool,
    pub used_by: Option<Uuid>,
    pub used_at: Option<DateTime<Utc>>,

    pub updated_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl InviteModel {
    pub fn new(created_by: Uuid, token: String, lifetime: Duration) -> ServerResult<Self> {
        let new_invite = NewInviteModel {
            created_by,
            token,
            expires_at: Utc::now() + lifetime,
        };

        let invite = diesel::insert_into(invites::table)
            .values(&new_invite)
            .get_result::<InviteModel>(&mut get_db()?)?;

        return Ok(invite);
    }

    pub fn find_by_token(token: &str) -> ServerResult<Option<Self>> {
        let invite = invites::table
            .filter(invites::token.eq(token))
            .first::<InviteModel>(&mut get_db()?)
            .optional()?;

        return Ok(invite);
    }

    pub fn use_invite(mut self, used_by: Uuid) -> ServerResult<()> {
        self.used = true;
        self.used_by = Some(used_by);
        self.used_at = Some(Utc::now());

        diesel::update(invites::table)
            .filter(invites::id.eq(self.id))
            .set(self)
            .get_result::<InviteModel>(&mut get_db()?)?;

        return Ok(());
    }
}
