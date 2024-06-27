-- Add down migration script here

DROP TRIGGER IF EXISTS trigger_update_at ON members;
DROP FUNCTION IF EXISTS on_update;
DROP TABLE IF EXISTS members;
DROP TYPE IF EXISTS status_member;
DROP TYPE IF EXISTS gender_member;
