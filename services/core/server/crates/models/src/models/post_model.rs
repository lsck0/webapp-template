use chrono::{DateTime, Utc};
use diesel::prelude::*;
use errors::ServerResult;
use uuid::Uuid;

use crate::{
    init::get_db,
    schema::{self, posts},
};

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = schema::posts)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewPostModel {
    pub title: String,
    pub content: String,
    pub author: Uuid,
}

#[derive(Debug, Clone, Queryable, Selectable, AsChangeset)]
#[diesel(table_name = schema::posts)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PostModel {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub author: Uuid,
    pub updated_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl PostModel {
    pub fn new(new_post: NewPostModel) -> ServerResult<Self> {
        let post = diesel::insert_into(posts::table)
            .values(&new_post)
            .get_result::<PostModel>(&mut get_db()?)?;

        return Ok(post);
    }

    pub fn get_all() -> ServerResult<Vec<Self>> {
        let posts = posts::table.load::<PostModel>(&mut get_db()?)?;

        return Ok(posts);
    }

    pub fn find_by_id(id: Uuid) -> ServerResult<Option<PostModel>> {
        let post = posts::table.find(id).first::<PostModel>(&mut get_db()?).optional()?;

        return Ok(post);
    }

    pub fn persist(self) -> ServerResult<Self> {
        let post = diesel::update(posts::table)
            .filter(posts::id.eq(self.id))
            .set(self)
            .get_result::<PostModel>(&mut get_db()?)?;

        return Ok(post);
    }

    pub fn delete(self) -> ServerResult<()> {
        diesel::delete(posts::table.filter(posts::id.eq(self.id))).execute(&mut get_db()?)?;

        return Ok(());
    }
}
