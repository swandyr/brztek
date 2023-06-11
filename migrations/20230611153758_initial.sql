-- Add migration script here
CREATE TABLE IF NOT EXISTS users (id INTEGER PRIMARY KEY);
CREATE TABLE IF NOT EXISTS guilds (id INTEGER PRIMARY KEY);
CREATE TABLE IF NOT EXISTS members (
    user_id,
    guild_id,
    role_color_id INTEGER,
    PRIMARY KEY (user_id, guild_id) FOREIGN KEY (user_id) REFERENCES users(id) FOREIGN KEY (guild_id) REFERENCES guilds(id)
);
CREATE TABLE IF NOT EXISTS levels (
    user_id INTEGER NOT NULL,
    guild_id INTEGER NOT NULL,
    xp INTEGER NOT NULL DEFAULT 0,
    level INTEGER NOT NULL DEFAULT 0,
    rank INTEGER NOT NULL DEFAULT 0,
    last_message INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (user_id, guild_id) FOREIGN KEY (user_id) REFERENCES users(id) FOREIGN KEY (guild_id) REFERENCES guilds(id)
);
CREATE TABLE IF NOT EXISTS learned_cmds (
    guild_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    content TEXT,
    PRIMARY KEY (guild_id, name) FOREIGN KEY (guild_id) REFERENCES guilds(id)
);
CREATE TABLE IF NOT EXISTS roulettes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    guild_id INTEGER NOT NULL,
    timestamp INTEGER NOT NULL,
    caller_id INTEGER NOT NULL,
    target_id INTEGER NOT NULL,
    rff_triggered INTEGER,
    FOREIGN KEY (guild_id) REFERENCES guilds(id)
);