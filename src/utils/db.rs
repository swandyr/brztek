use serenity::prelude::TypeMapKey;
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::sync::Arc;

use crate::utils::user_level::UserLevel;

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
            .await
            .expect("Cannot connect to database: {path}");
        Self { pool }
    }

    pub async fn run_migrations(&self) -> anyhow::Result<()> {
        sqlx::migrate!("./migrations").run(&self.pool).await?;
        Ok(())
    }

    /// Return Option<UserLevel> corresponding to `user_id` in the database.
    /// Return None if no entry with that user id is found.
    pub async fn get_user(&self, user_id: u64) -> anyhow::Result<UserLevel> {
        // Bit-cast user_id from u64 to i64, as SQLite does not support u64 integer
        let user_id = to_i64(user_id);

        let user_queried = sqlx::query!(
            "SELECT user_id, xp, level, messages, last_message FROM edn_ranks WHERE user_id = ?",
            user_id,
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(record) = user_queried {
            let record = [
                record.user_id,
                record.xp.unwrap_or_default(),
                record.level.unwrap_or_default(),
                record.messages.unwrap_or_default(),
                record.last_message.unwrap_or_default(),
            ];
            Ok(UserLevel::from(record))
        } else {
            // If no user is found, insert a new entry with user_id and default values.
            sqlx::query!("INSERT INTO edn_ranks (user_id) VALUES (?)", user_id,)
                .execute(&self.pool)
                .await?;
            Ok(UserLevel::new(user_id))
        }
    }

    /// Get an Option<UserLevel> by calling `get_user`.
    /// If None is returned, create a new `UserLevel` with the user id and the
    /// corresponding entry in the database.
    /// Call `levels::gain_xp` to update xp, messages and level of the user, then
    /// update the entry in the database.
    pub async fn update_user(&self, user: &UserLevel) -> anyhow::Result<()> {
        // Update user's entry in the database with new values.
        sqlx::query!(
            "UPDATE edn_ranks
                SET xp = ?,
                    level = ?,
                    messages = ?,
                    last_message = ?
                WHERE user_id = ?",
            user.xp,
            user.level,
            user.messages,
            user.last_message,
            user.user_id,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_all_users(&self) -> anyhow::Result<Vec<UserLevel>> {
        let all_users_queried =
            sqlx::query!("SELECT user_id, xp, level, messages, last_message FROM edn_ranks")
                .fetch_all(&self.pool)
                .await?;

        let all_users = all_users_queried
            .iter()
            .map(|record| {
                let params = [
                    record.user_id,
                    record.xp.unwrap_or_default(),
                    record.level.unwrap_or_default(),
                    record.messages.unwrap_or_default(),
                    record.last_message.unwrap_or_default(),
                ];

                UserLevel::from(params)
            })
            .collect();

        Ok(all_users)
    }

    /// Delete all rows in the table.
    pub async fn delete_table(&self) -> anyhow::Result<()> {
        sqlx::query!("DELETE FROM edn_ranks")
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}

/// Bit-cast u64 (user.id in Discord API) to i64 (stored in the `SQLite` database).
const fn to_i64(unsigned: u64) -> i64 {
    let bit_cast = unsigned.to_be_bytes();
    i64::from_be_bytes(bit_cast)
}

/// Bit-cast i64 (stored in `SQLite` database) to u64 (user.id in Discord API).
pub const fn from_i64(signed: i64) -> u64 {
    let bit_cast = signed.to_be_bytes();
    u64::from_be_bytes(bit_cast)
}
