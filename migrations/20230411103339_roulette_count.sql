-- Add migration script here
CREATE TABLE IF NOT EXISTS roulette_count (
    guild_id    INTEGER NOT NULL,
    time_stamp  INTEGER NOT NULL,
    caller_id INTEGER NOT NULL,
    target_id INTEGER NOT NULL,
    PRIMARY KEY (guild_id, time_stamp)
    FOREIGN KEY (guild_id) REFERENCES config(guild_id)
)