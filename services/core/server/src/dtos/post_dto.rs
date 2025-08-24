use chrono::{DateTime, Utc};
use errors::{FieldError, ServerError, ServerResult, Validate, bail, body_error, ensure, validate_string_length};
use models::models::{
    post_model::{NewPostModel, PostModel},
    user_model::UserModel,
};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;
use uuid::Uuid;

use super::Model;
use crate::dtos::{DTO, user_dto::UserDTO};

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[ts(export)]
pub struct PostDTO {
    #[schema(value_type = String)]
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub author: UserDTO,
    #[schema(value_type = String)]
    pub updated_at: DateTime<Utc>,
    #[schema(value_type = String)]
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[ts(export)]
pub struct NewPostDTO {
    pub title: String,
    pub content: String,
    #[schema(value_type = String)]
    pub author: Uuid,
}

impl Model for PostModel {
    type DTO = PostDTO;
}

impl DTO for PostDTO {
    type Model = PostModel;

    fn to_model(self) -> ServerResult<Self::Model> {
        return Ok(PostModel {
            id: self.id,
            title: self.title,
            content: self.content,
            author: self.author.id,
            updated_at: self.updated_at,
            created_at: self.created_at,
        });
    }

    fn from_model(model: Self::Model) -> ServerResult<Self> {
        let Some(author) = UserModel::find_by_id(model.author)? else {
            bail!(ServerError::NonExistentId(model.author.to_string()));
        };

        return Ok(Self {
            id: model.id,
            title: model.title,
            content: model.content,
            author: UserDTO::from_model(author)?,
            updated_at: model.updated_at,
            created_at: model.created_at,
        });
    }
}

impl Model for NewPostModel {
    type DTO = NewPostDTO;
}

impl DTO for NewPostDTO {
    type Model = NewPostModel;

    fn to_model(self) -> ServerResult<Self::Model> {
        return Ok(NewPostModel {
            title: self.title,
            content: self.content,
            author: self.author,
        });
    }

    fn from_model(model: Self::Model) -> ServerResult<Self> {
        return Ok(Self {
            title: model.title,
            content: model.content,
            author: model.author,
        });
    }
}

impl Validate for PostDTO {
    fn validate(&self) -> ServerResult<()> {
        let mut errors = vec![];

        validate_string_length!(errors, self, title, 1, 100);
        validate_string_length!(errors, self, content, 1, 255);

        ensure!(errors.is_empty(), body_error!(errors));

        return Ok(());
    }
}

impl Validate for NewPostDTO {
    fn validate(&self) -> ServerResult<()> {
        let mut errors = vec![];

        validate_string_length!(errors, self, title, 1, 100);
        validate_string_length!(errors, self, content, 1, 255);

        ensure!(errors.is_empty(), body_error!(errors));

        return Ok(());
    }
}
