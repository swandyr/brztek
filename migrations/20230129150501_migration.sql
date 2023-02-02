-- Add migration script here
CREATE TABLE IF NOT EXISTS edn_ranks (
    user_id     INTEGER NOT NULL,
    xp          INTEGER DEFAULT 0,
    level       INTEGER DEFAULT 0           
)