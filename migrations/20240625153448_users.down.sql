-- Add down migration script here

DROP TRIGGER IF EXISTS trigger_update_at ON users;
DROP FUNCTION IF EXISTS on_update;
DROP TABLE IF EXISTS users;
DROP TYPE IF EXISTS role_user;
