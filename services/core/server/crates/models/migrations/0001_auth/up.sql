CREATE TABLE IF NOT EXISTS users (
    id UUID NOT NULL PRIMARY KEY DEFAULT (uuid_generate_v4()),

    name TEXT NOT NULL UNIQUE,

    password_hash TEXT NOT NULL,

    otp_enabled BOOLEAN NOT NULL DEFAULT FALSE,
    otp_validated BOOLEAN NOT NULL DEFAULT FALSE,
    otp_secret TEXT,
    otp_url TEXT,
    otp_recovery_codes TEXT[] CHECK (array_position(otp_recovery_codes, null) IS NULL),

    permissions TEXT[] NOT NULL DEFAULT ARRAY[]::TEXT[] CHECK (array_position(permissions, null) IS NULL),
    permissions_forbidden TEXT[] NOT NULL DEFAULT ARRAY[]::TEXT[] CHECK (array_position(permissions_forbidden, null) IS NULL),

    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS sessions (
    id UUID NOT NULL PRIMARY KEY DEFAULT (uuid_generate_v4()),

    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    session_token TEXT NOT NULL UNIQUE,
    access_token TEXT NOT NULL UNIQUE,

    user_agent TEXT NOT NULL,
    ip_address INET NOT NULL,
    last_used TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),

    valid BOOLEAN NOT NULL DEFAULT TRUE,
    invalidated_at TIMESTAMP WITH TIME ZONE,
    invalidated_reason TEXT,

    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS encryption_keys (
    id UUID NOT NULL PRIMARY KEY DEFAULT (uuid_generate_v4()),

    key TEXT NOT NULL UNIQUE,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,

    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- This table is volatile, expired invites are automatically removed by the database.
CREATE TABLE IF NOT EXISTS invites (
    id UUID NOT NULL PRIMARY KEY DEFAULT (uuid_generate_v4()),

    created_by UUID NOT NULL REFERENCES users(id) ON DELETE SET NULL,
    token TEXT NOT NULL UNIQUE,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,

    used BOOLEAN NOT NULL DEFAULT FALSE,
    used_by UUID REFERENCES users(id) ON DELETE SET NULL,
    used_at TIMESTAMP WITH TIME ZONE,

    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- This table is volatile, old login attempts are automatically removed by the database.
CREATE TABLE IF NOT EXISTS login_attempts (
    id UUID NOT NULL PRIMARY KEY DEFAULT (uuid_generate_v4()),

    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    failed_counter INT NOT NULL DEFAULT 1,
    last_attempt TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),

    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- This table is volatile, expired restrictions are automatically removed by the database.
CREATE TABLE IF NOT EXISTS login_restrictions (
    id UUID NOT NULL PRIMARY KEY DEFAULT (uuid_generate_v4()),

    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    restricted_until TIMESTAMP WITH TIME ZONE NOT NULL,

    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS roles (
    id UUID NOT NULL PRIMARY KEY DEFAULT (uuid_generate_v4()),

    name TEXT NOT NULL UNIQUE,
    priority INT NOT NULL UNIQUE, -- higher means earlier in permission resolution
    permissions TEXT[] NOT NULL DEFAULT ARRAY[]::TEXT[] CHECK (array_position(permissions, null) IS NULL),
    permissions_forbidden TEXT[] NOT NULL DEFAULT ARRAY[]::TEXT[] CHECK (array_position(permissions_forbidden, null) IS NULL),

    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS user_roles (
    PRIMARY KEY (user_id, role_id),

    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role_id UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,

    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX users_name_idx ON users (name);
CREATE INDEX sessions_user_id_idx ON sessions (user_id);
CREATE INDEX sessions_token_idx ON sessions (session_token);
CREATE INDEX sessions_access_token_idx ON sessions (access_token);
CREATE INDEX login_attempts_user_id_idx ON login_attempts (user_id);
CREATE INDEX login_restrictions_user_id_idx ON login_restrictions (user_id);
CREATE INDEX invites_token_idx ON invites (token);
CREATE INDEX roles_name_idx ON roles (name);
CREATE INDEX user_roles_user_id_idx ON user_roles (user_id);

-- Remove expired login restrictions automatically.
CREATE OR REPLACE FUNCTION remove_expired_login_restrictions() RETURNS trigger AS $$
BEGIN
    DELETE FROM login_restrictions WHERE restricted_until < NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER remove_expired_login_restrictions_trigger AFTER INSERT ON login_restrictions
    EXECUTE FUNCTION remove_expired_login_restrictions();

-- Remove old login attempts counters automatically.
CREATE OR REPLACE FUNCTION reset_login_attempts() RETURNS trigger AS $$
BEGIN
    DELETE FROM login_attempts WHERE updated_at < NOW() - INTERVAL '1 hour';
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER reset_login_attempts_trigger AFTER INSERT ON login_attempts
    EXECUTE FUNCTION reset_login_attempts();

-- Close abandoned sessions automatically if they are older than 30 days.
CREATE OR REPLACE FUNCTION close_abandoned_sessions() RETURNS TRIGGER AS $$
BEGIN
    UPDATE sessions
    SET valid = FALSE, invalidated_at = NOW(), invalidated_reason = 'Abandoned'
    WHERE valid = TRUE
    AND last_used < NOW() - INTERVAL '30 days';
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER close_abandoned_sessions_trigger AFTER INSERT ON sessions
    EXECUTE FUNCTION close_abandoned_sessions();

-- Remove old invalidated sessions automatically if there are more than 100.
CREATE OR REPLACE FUNCTION remove_old_invalidated_sessions() RETURNS TRIGGER AS $$
BEGIN
    DELETE FROM sessions
    WHERE valid = FALSE
    AND id IN (
        SELECT id
        FROM sessions
        WHERE valid = FALSE
        ORDER BY created_at ASC
        LIMIT GREATEST((SELECT COUNT(*) FROM sessions WHERE valid = FALSE) - 100, 0)
    );
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER remove_old_invalidated_sessions_trigger AFTER INSERT ON sessions
    EXECUTE FUNCTION remove_old_invalidated_sessions();

-- Remove expired invites automatically if they are older than 30 days.
CREATE OR REPLACE FUNCTION remove_expired_invites() RETURNS TRIGGER AS $$
BEGIN
    DELETE FROM invites WHERE expires_at < NOW() - INTERVAL '30 days' AND used = FALSE;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER remove_expired_invites_trigger AFTER INSERT ON invites
    EXECUTE FUNCTION remove_expired_invites();
