-- Add migration script here
CREATE TABLE IF NOT EXISTS yt_sub (
    yt_channel_name TEXT NOT NULL,
    yt_channel_id TEXT NOT NULL,
    guild_id INTEGER NOT NULL,
    post_channel_id INTEGER NOT NULL,
    expire_on DATETIME NOT NULL,
    PRIMARY KEY (guild_id, yt_channel_id) FOREIGN KEY(guild_id) REFERENCES guilds(id)
);
