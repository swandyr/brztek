-- Add migration script here
DROP TABLE config;
CREATE TABLE IF NOT EXISTS config (
    guild_id        PRIMARY KEY,
    spam_delay      INTEGER NOT NULL DEFAULT 60,
    min_xp_gain     INTEGER NOT NULL DEFAULT 15,
    max_xp_gain     INTEGER NOT NULL DEFAULT 25
);