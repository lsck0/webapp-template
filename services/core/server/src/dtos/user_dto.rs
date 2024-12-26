use chrono::{DateTime, Utc};
use errors::{ServerError, ServerResult, bail};
use models::models::user_model::UserModel;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;
use uuid::Uuid;

use super::Model;
use crate::dtos::DTO;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[ts(export)]
pub struct UserDTO {
    #[schema(value_type = String)]
    pub id: Uuid,

    pub name: String,

    pub otp_enabled: bool,
    pub otp_validated: bool,

    #[schema(value_type = String)]
    pub updated_at: DateTime<Utc>,
    #[schema(value_type = String)]
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[ts(export)]
pub struct NewUserDTO {
    pub name: String,
    pub password: String,
}

impl Model for UserModel {
    type DTO = UserDTO;
}

impl DTO for UserDTO {
    type Model = UserModel;

    fn to_model(self) -> ServerResult<Self::Model> {
        let Some(user) = UserModel::find_by_id(self.id)? else {
            bail!(ServerError::NonExistentId(self.id.to_string()));
        };

        return Ok(UserModel {
            id: self.id,
            name: self.name,
            password_hash: user.password_hash,
            otp_enabled: self.otp_enabled,
            otp_validated: self.otp_validated,
            otp_secret: user.otp_secret,
            otp_url: user.otp_url,
            otp_recovery_codes: user.otp_recovery_codes,
            permissions: vec![],
            permissions_forbidden: vec![],
            updated_at: self.updated_at,
            created_at: self.created_at,
        });
    }

    fn from_model(model: Self::Model) -> ServerResult<Self> {
        return Ok(Self {
            id: model.id,
            name: model.name,
            otp_enabled: model.otp_enabled,
            otp_validated: model.otp_validated,
            updated_at: model.updated_at,
            created_at: model.created_at,
        });
    }
}
