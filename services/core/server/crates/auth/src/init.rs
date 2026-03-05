#![allow(clippy::needless_return)]

use config::ServerConfig;
use models::{
    models::user_model::{NewUserModel, UserModel},
    permissions::Permissions,
    role_model::{NewRoleModel, RoleModel},
};

use crate::{HashedPassword, keyring::KeyRing};

/// Initialize the auth subsystem: keyring, default roles/permissions, and admin user.
pub fn initialize_auth() {
    KeyRing::initialize();

    let config = ServerConfig::get();

    if !config.enable_default_user {
        return;
    }

    if UserModel::find_by_name(&config.default_user_name).is_ok_and(|user| user.is_some()) {
        return;
    }

    let admin_role = match RoleModel::find_by_name("Admin") {
        Ok(Some(role)) => role,
        _ => {
            let mut role = RoleModel::new(NewRoleModel {
                name: String::from("Admin"),
                priority: 100,
            })
            .expect("Failed to create the default admin role.");

            role.permissions = Permissions::all().into_iter().map(Some).collect();
            role.persist().expect("Failed to persist the default admin role.")
        }
    };

    let password_hash = HashedPassword::hash(&config.default_user_password)
        .expect("Failed to hash default user password")
        .take();

    let name_id = UserModel::assign_name_id(&config.default_user_name)
        .expect("Failed to assign name ID for default admin user");

    let admin_user = UserModel::new(NewUserModel {
        name: config.default_user_name,
        name_id,
        password_hash,
    })
    .expect("Failed to create the default admin user.");

    admin_user
        .add_role(admin_role.id)
        .expect("Failed to add the default admin role to the default admin user.");
}
