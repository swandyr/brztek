-- Add migration script here
CREATE TABLE IF NOT EXISTS config (
    guild_id        PRIMARY KEY,
    spam_delay      INTEGER DEFAULT 60,
    min_xp_gain     INTEGER DEFAULT 15,
    max_xp_gain     INTEGER DEFAULT 25
);