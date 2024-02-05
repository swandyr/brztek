-- Add migration script here
CREATE TABLE IF NOT EXISTS mention_roles (
  guild_id INTEGER NOT NULL,
  role_id INTEGER NOT NULL UNIQUE
);
