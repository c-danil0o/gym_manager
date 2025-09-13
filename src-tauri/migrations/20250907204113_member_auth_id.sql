-- Add migration script here
ALTER TABLE members ADD COLUMN auth_user_id TEXT DEFAULT NULL;
