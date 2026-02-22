use chrono::{DateTime, Utc};
use errors::{ServerError, ServerResult, bail};
use models::models::user_model::UserModel;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;
use uuid::Uuid;

use super::Model;
use crate::dtos::DTO;

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[ts(export)]
pub struct UserDTO {
    #[schema(value_type = String)]
    pub id: Uuid,

    pub name: String,

    #[schema(value_type = String)]
    pub updated_at: DateTime<Utc>,
    #[schema(value_type = String)]
    pub created_at: DateTime<Utc>,
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

        return Ok(user);
    }

    fn from_model(model: Self::Model) -> ServerResult<Self> {
        return Ok(Self {
            id: model.id,
            name: model.name,
            updated_at: model.updated_at,
            created_at: model.created_at,
        });
    }
}
