DROP INDEX IF EXISTS users_name_name_id_idx;
ALTER TABLE users DROP CONSTRAINT users_name_name_id_key;

ALTER TABLE users ADD CONSTRAINT users_name_key UNIQUE (name);
CREATE INDEX users_name_idx ON users (name);

ALTER TABLE users DROP COLUMN name_id;
