// @generated automatically by Diesel CLI.

diesel::table! {
    invites (id) {
        id -> Uuid,
        created_by -> Uuid,
        token -> Text,
        expires_at -> Timestamptz,
        used -> Bool,
        used_by -> Nullable<Uuid>,
        used_at -> Nullable<Timestamptz>,
        updated_at -> Timestamptz,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    login_attempts (id) {
        id -> Uuid,
        user_id -> Uuid,
        failed_counter -> Int4,
        last_attempt -> Timestamptz,
        updated_at -> Timestamptz,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    login_restrictions (id) {
        id -> Uuid,
        user_id -> Uuid,
        restricted_until -> Timestamptz,
        updated_at -> Timestamptz,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    posts (id) {
        id -> Uuid,
        title -> Text,
        content -> Text,
        author -> Uuid,
        updated_at -> Timestamptz,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    roles (id) {
        id -> Uuid,
        name -> Text,
        priority -> Int4,
        permissions -> Array<Nullable<Text>>,
        permissions_forbidden -> Array<Nullable<Text>>,
        updated_at -> Timestamptz,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    sessions (id) {
        id -> Uuid,
        user_id -> Uuid,
        session_token -> Text,
        access_token -> Text,
        user_agent -> Text,
        ip_address -> Inet,
        last_used -> Timestamptz,
        valid -> Bool,
        invalidated_at -> Nullable<Timestamptz>,
        invalidated_reason -> Nullable<Text>,
        updated_at -> Timestamptz,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    spatial_ref_sys (srid) {
        srid -> Int4,
        #[max_length = 256]
        auth_name -> Nullable<Varchar>,
        auth_srid -> Nullable<Int4>,
        #[max_length = 2048]
        srtext -> Nullable<Varchar>,
        #[max_length = 2048]
        proj4text -> Nullable<Varchar>,
    }
}

diesel::table! {
    user_roles (user_id, role_id) {
        user_id -> Uuid,
        role_id -> Uuid,
        updated_at -> Timestamptz,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        name -> Text,
        password_hash -> Text,
        otp_enabled -> Bool,
        otp_validated -> Bool,
        otp_secret -> Nullable<Text>,
        otp_url -> Nullable<Text>,
        otp_recovery_codes -> Nullable<Array<Nullable<Text>>>,
        permissions -> Array<Nullable<Text>>,
        permissions_forbidden -> Array<Nullable<Text>>,
        updated_at -> Timestamptz,
        created_at -> Timestamptz,
    }
}

diesel::joinable!(login_attempts -> users (user_id));
diesel::joinable!(login_restrictions -> users (user_id));
diesel::joinable!(posts -> users (author));
diesel::joinable!(sessions -> users (user_id));
diesel::joinable!(user_roles -> roles (role_id));
diesel::joinable!(user_roles -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    invites,
    login_attempts,
    login_restrictions,
    posts,
    roles,
    sessions,
    spatial_ref_sys,
    user_roles,
    users,
);
