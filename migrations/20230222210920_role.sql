-- Add migration script here
CREATE TABLE IF NOT EXISTS role_color (
    guild_id    INTEGER NOT NULL,
    user_id     INTEGER NOT NULL,
    role_id     INTEGER,
    PRIMARY KEY (user_id, guild_id)
    FOREIGN KEY (guild_id, user_id) REFERENCES levels(guild_id, user_id)
);