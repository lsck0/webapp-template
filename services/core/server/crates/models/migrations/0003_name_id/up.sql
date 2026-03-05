ALTER TABLE users ADD COLUMN name_id SMALLINT NOT NULL DEFAULT 0;

ALTER TABLE users DROP CONSTRAINT users_name_key;
DROP INDEX IF EXISTS users_name_idx;

ALTER TABLE users ADD CONSTRAINT users_name_name_id_key UNIQUE (name, name_id);
CREATE INDEX users_name_name_id_idx ON users (name, name_id);
