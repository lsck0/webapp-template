use chrono::{DateTime, Utc};
use errors::{ServerError, ServerResult, bail};
use ipnetwork::IpNetwork;
use models::models::{
    session_model::{SessionInvalidationReason, SessionModel},
    user_model::UserModel,
};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;
use uuid::Uuid;

use super::{DTO, Model};
use crate::dtos::user_dto::UserDTO;

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[ts(export)]
pub struct SessionDTO {
    #[schema(value_type = String)]
    pub id: Uuid,

    pub user: UserDTO,
    pub session_token: String,
    pub access_token: String,

    pub user_agent: String,
    #[schema(value_type = String)]
    #[ts(type = "string")]
    pub ip_address: IpNetwork,

    pub valid: bool,
    #[schema(value_type = Option<String>)]
    pub invalidated_at: Option<DateTime<Utc>>,
    pub invalidated_reason: Option<SessionInvalidationReason>,

    #[schema(value_type = String)]
    pub last_used: DateTime<Utc>,
    #[schema(value_type = String)]
    pub updated_at: DateTime<Utc>,
    #[schema(value_type = String)]
    pub created_at: DateTime<Utc>,
}

impl Model for SessionModel {
    type DTO = SessionDTO;
}

impl DTO for SessionDTO {
    type Model = SessionModel;

    fn to_model(self) -> ServerResult<Self::Model> {
        return Ok(SessionModel {
            id: self.id,
            user_id: self.user.id,
            session_token: self.session_token,
            access_token: self.access_token,
            user_agent: self.user_agent,
            ip_address: self.ip_address,
            valid: self.valid,
            invalidated_at: self.invalidated_at,
            invalidated_reason: self.invalidated_reason,
            last_used: self.last_used,
            updated_at: self.updated_at,
            created_at: self.created_at,
        });
    }

    fn from_model(model: Self::Model) -> ServerResult<Self> {
        let Some(user) = UserModel::find_by_id(model.user_id)? else {
            bail!(ServerError::NonExistentId(model.user_id.to_string()));
        };

        return Ok(Self {
            id: model.id,
            user: UserDTO::from_model(user)?,
            session_token: model.session_token,
            access_token: model.access_token,
            user_agent: model.user_agent,
            ip_address: model.ip_address,
            valid: model.valid,
            invalidated_at: model.invalidated_at,
            invalidated_reason: model.invalidated_reason,
            last_used: model.last_used,
            updated_at: model.updated_at,
            created_at: model.created_at,
        });
    }
}
