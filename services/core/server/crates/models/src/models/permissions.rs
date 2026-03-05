use diesel::expression::AsExpression;
use macros::PgText;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter, EnumString, IntoEnumIterator};
use ts_rs::TS;
use utoipa::ToSchema;

#[derive(
    AsExpression,
    Clone,
    Copy,
    Debug,
    Deserialize,
    Display,
    EnumIter,
    EnumString,
    Eq,
    Hash,
    PartialEq,
    PgText,
    Serialize,
    TS,
    ToSchema,
)]
#[diesel(sql_type = diesel::sql_types::Text)]
#[ts(export)]
pub enum Permissions {
    CanInvite,
    CanCreatePost,
    CanEditOwnPost,
    CanEditAnyPost,
    CanDeleteOwnPost,
    CanDeleteAnyPost,
    CanViewUsers,
    CanEditUsers,
    CanDeleteUsers,
    CanManageRoles,
    CanManageSessions,
}

impl Permissions {
    pub fn none() -> Vec<Self> {
        return vec![];
    }

    pub fn all() -> Vec<Self> {
        return Permissions::iter().collect();
    }
}
