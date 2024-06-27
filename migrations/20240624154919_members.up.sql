-- Add up migration script here
-- up.sql

SET TIME ZONE 'Asia/Jakarta';

DROP TABLE IF EXISTS members;
DROP TYPE IF EXISTS status_member;
DROP TYPE IF EXISTS gender_member;

CREATE TYPE status_member AS ENUM ('pekerja', 'iburumahtangga', 'pelajar', 'mahasiswa', 'pengangguran');
CREATE TYPE gender_member as ENUM ('lakilaki', 'perempuan');

CREATE TABLE members (
	id SERIAL PRIMARY KEY,
	nik INT NOT NULL UNIQUE,
	nama VARCHAR(255) NOT NULL,
	umur INT NOT NULL,
	tanggal_lahir DATE NOT NULL,
	tempat_lahir VARCHAR(255) NOT NULL,
	status status_member DEFAULT 'pengangguran' NOT NULL,
	gender gender_member NOT NULL,
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
BEFORE UPDATE ON members
FOR EACH ROW
EXECUTE FUNCTION on_update();
