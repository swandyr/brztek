use serenity::prelude::TypeMapKey;
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::sync::Arc;
use tracing::debug;

use crate::utils::levels::user_level::UserLevel;

pub struct Db {
    pool: SqlitePool,
}

impl TypeMapKey for Db {
    type Value = Arc<Self>;
}

impl Db {
    pub async fn new(db_path: &str) -> Self {
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(db_path)
            // .connect_with(
            //     sqlx::sqlite::SqliteConnectOptions::new()
            //         .filename("database.sqlite")
            //         .create_if_missing(true),
            // )
            .await
            .expect("Cannot connect to database: {path}");
        Self { pool }
    }

    pub async fn run_migrations(&self) -> anyhow::Result<()> {
        sqlx::migrate!("./migrations").run(&self.pool).await?;
        Ok(())
    }

    /// Return `UserLevel` corresponding to `user_id` in the database.
    ///
    /// If no user is found, create a new entry with `user_id` and returns
    /// new `UserLevel`.
    pub async fn get_user(&self, user_id: u64, guild_id: u64) -> anyhow::Result<UserLevel> {
        // Bit-cast `user_id` from u64 to i64, as SQLite does not support u64 integer
        let user_id = to_i64(user_id);
        let guild_id = to_i64(guild_id);

        let user_queried = sqlx::query!(
            "SELECT user_id, xp, level, rank, messages, last_message FROM levels 
            WHERE user_id = ? 
            AND guild_id = ?",
            user_id,
            guild_id,
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(record) = user_queried {
            let user = (
                from_i64(record.user_id),
                record.xp.unwrap_or_default(),
                record.level.unwrap_or_default(),
                record.rank.unwrap_or_default(),
                record.messages.unwrap_or_default(),
                record.last_message.unwrap_or_default(),
            );
            Ok(UserLevel::from(user))
        } else {
            sqlx::query!(
                "INSERT INTO levels (user_id, guild_id) VALUES (?, ?)",
                user_id,
                guild_id
            )
            .execute(&self.pool)
            .await?;
            Ok(UserLevel::new(from_i64(user_id)))
        }
    }

    /// Update user's entry in the database with new values.
    pub async fn update_user(&self, user: &UserLevel, guild_id: u64) -> anyhow::Result<()> {
        // Bit-cast `user_id` from u64 to i64, as SQLite does not support u64 integer
        let user_id = to_i64(user.user_id);
        let guild_id = to_i64(guild_id);

        sqlx::query!(
            "UPDATE levels
                SET xp = ?,
                    level = ?,
                    messages = ?,
                    last_message = ?
                WHERE user_id = ?
                AND guild_id = ?",
            user.xp,
            user.level,
            user.messages,
            user.last_message,
            user_id,
            guild_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // Update user rank in the database
    pub async fn update_ranks(&self, users: &Vec<UserLevel>, guild_id: u64) -> anyhow::Result<()> {
        let guild_id = to_i64(guild_id);

        for user in users {
            let user_id = to_i64(user.user_id);

            sqlx::query!(
                "UPDATE levels
                SET rank = ?
                WHERE user_id = ?
                AND guild_id = ?",
                user.rank,
                user_id,
                guild_id
            )
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    /// Get all entries in the dabase and returns a `Vec<UserLevel>`
    pub async fn get_all_users(&self, guild_id: u64) -> anyhow::Result<Vec<UserLevel>> {
        let guild_id = to_i64(guild_id);

        let all_users_queried = sqlx::query!(
            "SELECT user_id, xp, level, rank, messages, last_message FROM levels
            WHERE guild_id = ?",
            guild_id
        )
        .fetch_all(&self.pool)
        .await?;

        let all_users = all_users_queried
            .iter()
            .map(|record| {
                let params = (
                    from_i64(record.user_id),
                    record.xp.unwrap_or_default(),
                    record.level.unwrap_or_default(),
                    record.rank.unwrap_or_default(),
                    record.messages.unwrap_or_default(),
                    record.last_message.unwrap_or_default(),
                );

                UserLevel::from(params)
            })
            .collect();

        Ok(all_users)
    }

    /// Delete all rows in the table.
    pub async fn delete_table(&self, guild_id: u64) -> anyhow::Result<()> {
        let guild_id = to_i64(guild_id);

        sqlx::query!("DELETE FROM levels WHERE guild_id = ?", guild_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn get_command(&self, command_name: &str) -> anyhow::Result<Option<String>> {
        let content = sqlx::query!(
            "SELECT content FROM learned_cmd WHERE name = ?",
            command_name
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(record) = content {
            debug!("command name: {command_name}");
            debug!("command content: {:?}", record.content);
            Ok(record.content)
        } else {
            Ok(None)
        }
    }

    pub async fn learn_command(&self, command_name: &str, content: &str) -> anyhow::Result<()> {
        sqlx::query!(
            "INSERT INTO learned_cmd (name, content) VALUES (?, ?) 
            ON CONFLICT (name) DO UPDATE SET content = ?",
            command_name,
            content,
            content
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // #[allow(dead_code)]
    // pub async fn get_user_as(&self, user_id: u64) -> anyhow::Result<Option<UserLevel>> {
    //     let user_id = to_i64(user_id);

    //     let user = sqlx::query_as!(
    //         UserLevel,
    //         "SELECT * FROM (select (1) as user_id, (2) as xp, (3) as level) edn_ranks WHERE user_id = ?",
    //         user_id,
    //     )
    //     .fetch_optional(&self.pool)
    //     .await?;

    //     Ok(user)
    // }
}

/// Bit-cast u64 (user.id in Discord API) to i64 (stored in the `SQLite` database).
const fn to_i64(unsigned: u64) -> i64 {
    let bit_cast = unsigned.to_be_bytes();
    i64::from_be_bytes(bit_cast)
}

/// Bit-cast i64 (stored in `SQLite` database) to u64 (user.id in Discord API).
const fn from_i64(signed: i64) -> u64 {
    let bit_cast = signed.to_be_bytes();
    u64::from_be_bytes(bit_cast)
}
