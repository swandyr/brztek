use chrono::Utc;
use serenity::prelude::TypeMapKey;
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::sync::Arc;

use crate::utils::{
    levels::{self, ANTI_SPAM_DELAY},
    user_level::UserLevel,
};

pub struct Db {
    pool: SqlitePool,
}

impl TypeMapKey for Db {
    type Value = Arc<Db>;
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
        Db { pool }
    }

    pub async fn run_migrations(&self) -> anyhow::Result<()> {
        sqlx::migrate!("./migrations").run(&self.pool).await?;
        Ok(())
    }

    /// Get an Option<UserLevel> by calling get_user.
    /// If None is returned, create a new UserLevel with the user_id and the
    /// corresponding entry in the database.
    /// Call levels::gain_xp to update xp, messages and level of the user, then
    /// update the entry in the database.
    pub async fn add_user_xp(&self, user_id: u64) -> anyhow::Result<bool> {
        // Retrieve the user's data from database
        let queried_user = self.get_user(user_id).await?;

        // If no user is found, insert a new entry with user_id and default values.
        let mut user = match queried_user {
            Some(u) => u,
            None => {
                let user_id = to_i64(user_id);
                sqlx::query!(
                    "INSERT INTO edn_ranks (user_id, xp) VALUES (?, ?)",
                    user_id,
                    0,
                )
                .execute(&self.pool)
                .await?;
                UserLevel::new(user_id)
            }
        };

        // Check the time between last and new message.
        // If time is below anti spam constant, return early
        // without adding xp.
        let now: i64 = Utc::now().timestamp();
        if now - user.last_message < ANTI_SPAM_DELAY {
            return Ok(false);
        }

        // levels::gain_xp(&mut user) adds xp, increments messages count
        // and level (if xp requirements is met).
        // level_up is a bool returned if user.level has been incremented.
        let level_up = levels::gain_xp(&mut user);

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
            now,
            user.user_id,
        )
        .execute(&self.pool)
        .await?;

        Ok(level_up)
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

    /// Return Option<UserLevel> corresponding to user_id in the database.
    /// Return None if no entry with that user_id is found.
    pub async fn get_user(&self, user_id: u64) -> anyhow::Result<Option<UserLevel>> {
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
                record.xp.unwrap(),
                record.level.unwrap(),
                record.messages.unwrap(),
                record.last_message.unwrap(),
            ];
            Ok(Some(UserLevel::from(record)))
        } else {
            Ok(None)
        }
    }

    pub async fn get_all_users(&self) -> anyhow::Result<Vec<UserLevel>> {
        let all_users_queried =
            sqlx::query!("SELECT user_id, xp, level, messages, last_message FROM edn_ranks")
                .fetch_all(&self.pool)
                .await
                .unwrap();

        let all_users = all_users_queried
            .iter()
            .map(|record| {
                let params = [
                    record.user_id,
                    record.xp.unwrap_or(0),
                    record.level.unwrap_or(0),
                    record.messages.unwrap_or(0),
                    record.last_message.unwrap_or(0),
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

/// Bit-cast u64 (user.id in Discord API) to i64 (stored in the SQLite database).
fn to_i64(unsigned: u64) -> i64 {
    let bit_cast = unsigned.to_be_bytes();
    i64::from_be_bytes(bit_cast)
}

/// Bit-cast i64 (stored in SQLite database) to u64 (user.id in Discord API).
pub fn from_i64(signed: i64) -> u64 {
    let bit_cast = signed.to_be_bytes();
    u64::from_be_bytes(bit_cast)
}
