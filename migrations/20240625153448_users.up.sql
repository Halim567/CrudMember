-- Add up migration script here
-- up.sql

SET TIME ZONE 'Asia/Jakarta';

DROP TABLE IF EXISTS users;
DROP TYPE IF EXISTS role_user;

CREATE TYPE role_user AS ENUM ('admin', 'user');

CREATE TABLE users (
	id SERIAL PRIMARY KEY,
	email VARCHAR(255) NOT NULL UNIQUE,
	password VARCHAR(255) NOT NULL,
	role role_user DEFAULT 'user' NOT NULL,
	create_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
	update_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL
);


CREATE OR REPLACE FUNCTION on_update()
RETURNS TRIGGER AS $$
BEGIN
    NEW.update_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_update_at
BEFORE UPDATE ON users
FOR EACH ROW
EXECUTE FUNCTION on_update();