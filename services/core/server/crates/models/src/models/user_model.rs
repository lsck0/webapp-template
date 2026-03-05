use std::collections::HashSet;

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use errors::ServerResult;
use rand::RngExt;
use uuid::Uuid;

use crate::{
    init::get_db,
    permissions::Permissions,
    role_model::{NewUserRoleModel, RoleModel},
    schema::{self, users},
};

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewUserModel {
    pub name: String,
    pub name_id: i16,
    pub password_hash: String,
}

#[derive(Debug, Clone, AsChangeset, Queryable, Selectable)]
#[diesel(table_name = schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UserModel {
    pub id: Uuid,

    pub name: String,
    pub name_id: i16,

    pub password_hash: String,

    pub otp_enabled: bool,
    pub otp_validated: bool,
    pub otp_secret: Option<String>,
    pub otp_url: Option<String>,
    pub otp_recovery_codes: Option<Vec<Option<String>>>,

    pub permissions: Vec<Option<Permissions>>,
    pub permissions_forbidden: Vec<Option<Permissions>>,

    pub updated_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl UserModel {
    pub fn new(new_user: NewUserModel) -> ServerResult<Self> {
        let user = diesel::insert_into(users::table)
            .values(&new_user)
            .get_result::<UserModel>(&mut get_db()?)?;

        return Ok(user);
    }

    /// Assign a random available name_id for the given name.
    pub fn assign_name_id(name: &str) -> ServerResult<i16> {
        let taken: Vec<i16> = users::table
            .filter(users::name.eq(name))
            .select(users::name_id)
            .load(&mut get_db()?)?;

        let taken_set: HashSet<i16> = taken.into_iter().collect();

        // Try random first for speed
        let mut rng = rand::rng();
        for _ in 0..50 {
            let candidate = rng.random_range(0..10000);
            if !taken_set.contains(&candidate) {
                return Ok(candidate);
            }
        }

        // Fallback to sequential scan
        for candidate in 0..10000 {
            if !taken_set.contains(&candidate) {
                return Ok(candidate);
            }
        }

        return Err(errors::ServerError::Internal(
            "No available name ID for this name".into(),
        ));
    }

    pub fn get_all() -> ServerResult<Vec<Self>> {
        let users = users::table.load::<UserModel>(&mut get_db()?)?;

        return Ok(users);
    }

    pub fn find_by_id(id: Uuid) -> ServerResult<Option<Self>> {
        let user = users::table.find(id).first::<UserModel>(&mut get_db()?).optional()?;

        return Ok(user);
    }

    pub fn find_by_name(name: &str) -> ServerResult<Option<Self>> {
        let user = users::table
            .filter(users::name.eq(name))
            .first::<UserModel>(&mut get_db()?)
            .optional()?;

        return Ok(user);
    }

    pub fn find_by_tag(name: &str, name_id: i16) -> ServerResult<Option<Self>> {
        let user = users::table
            .filter(users::name.eq(name).and(users::name_id.eq(name_id)))
            .first::<UserModel>(&mut get_db()?)
            .optional()?;

        return Ok(user);
    }

    /// Format the user tag as `name#0000`.
    pub fn tag(&self) -> String {
        return format!("{}#{:04}", self.name, self.name_id);
    }

    pub fn persist(self) -> ServerResult<Self> {
        let user = diesel::update(users::table.find(self.id))
            .set(self)
            .get_result::<UserModel>(&mut get_db()?)?;

        return Ok(user);
    }

    pub fn add_role(self, role_id: Uuid) -> ServerResult<()> {
        let new_user_role = NewUserRoleModel {
            user_id: self.id,
            role_id,
        };

        diesel::insert_into(schema::user_roles::table)
            .values(&new_user_role)
            .execute(&mut get_db()?)?;

        return Ok(());
    }

    pub fn delete(id: Uuid) -> ServerResult<()> {
        diesel::delete(users::table.find(id)).execute(&mut get_db()?)?;

        return Ok(());
    }

    /// Get all roles assigned to this user, ordered by priority (highest first).
    pub fn get_roles(&self) -> ServerResult<Vec<RoleModel>> {
        let roles = schema::user_roles::table
            .inner_join(schema::roles::table.on(schema::roles::id.eq(schema::user_roles::role_id)))
            .filter(schema::user_roles::user_id.eq(self.id))
            .select(RoleModel::as_select())
            .order(schema::roles::priority.desc())
            .load::<RoleModel>(&mut get_db()?)?;

        return Ok(roles);
    }

    /// Resolve the effective permissions for this user.
    ///
    /// Resolution order (highest priority first):
    /// 1. User-level forbidden permissions (always denied)
    /// 2. User-level granted permissions (always granted, unless forbidden)
    /// 3. Role permissions, processed by priority (highest first):
    ///    - Role forbidden permissions remove from the set
    ///    - Role granted permissions add to the set
    pub fn resolve_permissions(&self) -> ServerResult<HashSet<Permissions>> {
        let mut effective: HashSet<Permissions> = HashSet::new();

        // Start with role permissions (lowest priority first, so higher overrides)
        let mut roles = self.get_roles()?;
        roles.reverse(); // process lowest priority first

        for role in &roles {
            for perm in role.permissions.iter().flatten() {
                effective.insert(*perm);
            }
            for perm in role.permissions_forbidden.iter().flatten() {
                effective.remove(perm);
            }
        }

        // User-level grants override roles
        for perm in self.permissions.iter().flatten() {
            effective.insert(*perm);
        }

        // User-level forbidden always wins
        for perm in self.permissions_forbidden.iter().flatten() {
            effective.remove(perm);
        }

        return Ok(effective);
    }
}
