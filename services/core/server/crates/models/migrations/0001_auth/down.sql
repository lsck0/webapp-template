DROP TRIGGER IF EXISTS remove_old_invalidated_sessions_trigger ON sessions;
DROP FUNCTION IF EXISTS remove_old_invalidated_sessions;

DROP TRIGGER IF EXISTS close_abandoned_sessions_trigger ON sessions;
DROP FUNCTION IF EXISTS close_abandoned_sessions;

DROP TRIGGER IF EXISTS remove_expired_login_restrictions_trigger ON login_restrictions;
DROP FUNCTION IF EXISTS remove_expired_login_restrictions;

DROP TRIGGER IF EXISTS reset_login_attempts_trigger ON login_attempts;
DROP FUNCTION IF EXISTS reset_login_attempts;

DROP TRIGGER IF EXISTS remove_expired_invites_trigger ON invites;
DROP FUNCTION IF EXISTS remove_expired_invites;

DROP INDEX IF EXISTS users_name_idx;
DROP INDEX IF EXISTS sessions_user_id_idx;
DROP INDEX IF EXISTS sessions_token_idx;
DROP INDEX IF EXISTS sessions_access_token_idx;
DROP INDEX IF EXISTS login_attempts_user_id_idx;
DROP INDEX IF EXISTS login_restrictions_user_id_idx;
DROP INDEX IF EXISTS invites_token_idx;
DROP INDEX IF EXISTS roles_name_idx;
DROP INDEX IF EXISTS user_roles_user_id_idx;

DROP TABLE IF EXISTS sessions;
DROP TABLE IF EXISTS login_restrictions;
DROP TABLE IF EXISTS login_attempts;
DROP TABLE IF EXISTS invites;

DROP TABLE IF EXISTS user_roles;
DROP TABLE IF EXISTS roles;

DROP TABLE IF EXISTS users;
