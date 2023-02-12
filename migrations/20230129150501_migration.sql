-- Add migration script here
CREATE TABLE IF NOT EXISTS levels (
    user_id         INTEGER NOT NULL,
    guild_id        INTEGER NOT NULL,
    xp              INTEGER DEFAULT 0,
    level           INTEGER DEFAULT 0,
    rank            INTEGER DEFAULT 0,
    messages        INTEGER DEFAULT 0,
    last_message    INTEGER DEFAULT 0,
    PRIMARY KEY (user_id, guild_id)
);
CREATE TABLE IF NOT EXISTS learned_cmd (
    name            TEXT PRIMARY KEY,
    content         TEXT
);