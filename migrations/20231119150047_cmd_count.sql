-- Add migration script here
CREATE TABLE IF NOT EXISTS cmd_count (
  guild_id INTEGER NOT NULL,
  command TEXT NOT NULL,
  count INTEGER DEFAULT 0,
  PRIMARY KEY (guild_id, command)
);
