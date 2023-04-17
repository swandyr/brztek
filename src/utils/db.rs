use serenity::prelude::TypeMapKey;
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::sync::Arc;
use tracing::{debug, instrument};

use crate::levels::user_level::UserLevel;

#[derive(Debug)]
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

    #[instrument]
    pub async fn run_migrations(&self) -> anyhow::Result<()> {
        sqlx::migrate!("./migrations").run(&self.pool).await?;
        Ok(())
    }

    /// Import levels from Mee6.
    ///
    /// Clear all users entries corresponding to the guild_id first,
    /// and insert all new entries in hte `uers: Vec<UserLevel>
    #[instrument]
    pub async fn import_from_mee6(
        &self,
        users: Vec<UserLevel>,
        guild_id: u64,
    ) -> anyhow::Result<()> {
        self.delete_table(guild_id).await?;

        let guild_id = to_i64(guild_id);

        for user in users {
            let user_id = to_i64(user.user_id);
            sqlx::query!(
                "INSERT INTO levels (user_id, guild_id, xp, level, rank, last_message)
                VALUES (?, ?, ?, ?, ?, ?)",
                user_id,
                guild_id,
                user.xp,
                user.level,
                user.rank,
                user.last_message,
            )
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    /// Return `UserLevel` corresponding to `user_id` in the database.
    ///
    /// If no user is found, create a new entry with `user_id` and returns
    /// new `UserLevel`.
    #[instrument]
    pub async fn get_user(&self, user_id: u64, guild_id: u64) -> anyhow::Result<UserLevel> {
        // Bit-cast `user_id` from u64 to i64, as SQLite does not support u64 integer
        let user_id = to_i64(user_id);
        let guild_id = to_i64(guild_id);

        let response = sqlx::query!(
            "SELECT user_id, xp, level, rank, last_message FROM levels 
            WHERE user_id = ? AND guild_id = ?",
            user_id,
            guild_id,
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(record) = response {
            let user = (
                from_i64(record.user_id),
                record.xp.unwrap_or_default(),
                record.level.unwrap_or_default(),
                record.rank.unwrap_or_default(),
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
    #[instrument]
    pub async fn update_user(&self, user: &UserLevel, guild_id: u64) -> anyhow::Result<()> {
        // Bit-cast `user_id` from u64 to i64, as SQLite does not support u64 integer
        let user_id = to_i64(user.user_id);
        let guild_id = to_i64(guild_id);

        sqlx::query!(
            "UPDATE levels
            SET xp = ?, level = ?, last_message = ?
            WHERE user_id = ? AND guild_id = ?",
            user.xp,
            user.level,
            user.last_message,
            user_id,
            guild_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Update user rank in the database
    #[instrument]
    pub async fn update_ranks(&self, users: Vec<UserLevel>, guild_id: u64) -> anyhow::Result<()> {
        let guild_id = to_i64(guild_id);

        for user in users {
            let user_id = to_i64(user.user_id);

            sqlx::query!(
                "UPDATE levels
                SET rank = ?
                WHERE user_id = ? AND guild_id = ?",
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
    #[instrument]
    pub async fn get_all_users(&self, guild_id: u64) -> anyhow::Result<Vec<UserLevel>> {
        let guild_id = to_i64(guild_id);

        let response = sqlx::query!(
            "SELECT user_id, xp, level, rank, last_message FROM levels
            WHERE guild_id = ?",
            guild_id
        )
        .fetch_all(&self.pool)
        .await?;

        let all_users = response
            .iter()
            .map(|record| {
                let params = (
                    from_i64(record.user_id),
                    record.xp.unwrap_or_default(),
                    record.level.unwrap_or_default(),
                    record.rank.unwrap_or_default(),
                    record.last_message.unwrap_or_default(),
                );

                UserLevel::from(params)
            })
            .collect();

        Ok(all_users)
    }

    /// Delete all rows in the table.
    #[instrument]
    pub async fn delete_table(&self, guild_id: u64) -> anyhow::Result<()> {
        let guild_id = to_i64(guild_id);

        sqlx::query!("DELETE FROM levels WHERE guild_id = ?", guild_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    ////////////////////////////////////////////////////////////////////////////////////////

    #[instrument]
    pub async fn set_role_color(
        &self,
        guild_id: u64,
        user_id: u64,
        role_id: u64,
    ) -> anyhow::Result<()> {
        let guild_id = to_i64(guild_id);
        let user_id = to_i64(user_id);
        let role_id = to_i64(role_id);

        sqlx::query!(
            "INSERT INTO role_color (guild_id, user_id, role_id) VALUES (?, ?, ?)
            ON CONFLICT (guild_id, user_id) DO UPDATE SET role_id = ?",
            guild_id,
            user_id,
            role_id,
            role_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    #[instrument]
    pub async fn get_role_color(&self, guild_id: u64, user_id: u64) -> anyhow::Result<Option<u64>> {
        let guild_id = to_i64(guild_id);
        let user_id = to_i64(user_id);

        let response = sqlx::query!(
            "SELECT role_id FROM role_color WHERE guild_id = ? AND user_id = ?",
            guild_id,
            user_id
        )
        .fetch_optional(&self.pool)
        .await?;

        let role_id = response.and_then(|record| record.role_id.map(from_i64));

        Ok(role_id)
    }

    ///////////////////////////////////////////////////////////////////////////////////////:

    #[instrument]
    pub async fn create_config_entry(&self, guild_id: u64) -> anyhow::Result<()> {
        let guild_id = to_i64(guild_id);

        sqlx::query!(
            "INSERT OR IGNORE INTO config (guild_id) VALUES (?)",
            guild_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_xpsettings(&self, guild_id: u64) -> anyhow::Result<(i64, i64, i64)> {
        let guild_id = to_i64(guild_id);

        let record = sqlx::query!(
            "SELECT spam_delay, min_xp_gain, max_xp_gain FROM config
            WHERE guild_id = ?",
            guild_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok((record.spam_delay, record.min_xp_gain, record.max_xp_gain))
    }

    #[instrument]
    pub async fn get_spam_delay(&self, guild_id: u64) -> anyhow::Result<i64> {
        let guild_id = to_i64(guild_id);

        let record = sqlx::query!(
            "SELECT spam_delay FROM config 
            WHERE guild_id = ?",
            guild_id,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(record.spam_delay)
    }

    #[instrument]
    pub async fn set_spam_delay(&self, guild_id: u64, value: i64) -> anyhow::Result<()> {
        let guild_id = to_i64(guild_id);

        sqlx::query!(
            "UPDATE config
            SET spam_delay = ?
            WHERE guild_id = ?",
            value,
            guild_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    #[instrument]
    pub async fn get_min_xp_gain(&self, guild_id: u64) -> anyhow::Result<i64> {
        let guild_id = to_i64(guild_id);

        let record = sqlx::query!(
            "SELECT min_xp_gain FROM config
            WHERE guild_id = ?",
            guild_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(record.min_xp_gain)
    }

    #[instrument]
    pub async fn set_min_xp_gain(&self, guild_id: u64, value: i64) -> anyhow::Result<()> {
        let guild_id = to_i64(guild_id);

        sqlx::query!(
            "UPDATE config
            SET min_xp_gain = ?
            WHERE guild_id = ?",
            value,
            guild_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    #[instrument]
    pub async fn get_max_xp_gain(&self, guild_id: u64) -> anyhow::Result<i64> {
        let guild_id = to_i64(guild_id);

        let record = sqlx::query!(
            "SELECT max_xp_gain FROM config
            WHERE guild_id = ?",
            guild_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(record.max_xp_gain)
    }

    #[instrument]
    pub async fn set_max_xp_gain(&self, guild_id: u64, value: i64) -> anyhow::Result<()> {
        let guild_id = to_i64(guild_id);

        sqlx::query!(
            "UPDATE config
            SET max_xp_gain = ?
            WHERE guild_id = ?",
            value,
            guild_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    #[instrument]
    pub async fn set_pub_channel_id(&self, channel_id: u64, guild_id: u64) -> anyhow::Result<()> {
        let channel_id = to_i64(channel_id);
        let guild_id = to_i64(guild_id);

        sqlx::query!(
            "UPDATE config
            SET pub_channel_id = ?
            WHERE guild_id = ?",
            channel_id,
            guild_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    #[instrument]
    pub async fn get_pub_channel_id(&self, guild_id: u64) -> anyhow::Result<Option<u64>> {
        let guild_id = to_i64(guild_id);

        let response = sqlx::query!(
            "SELECT pub_channel_id FROM config
            WHERE guild_id = ?",
            guild_id
        )
        .fetch_optional(&self.pool)
        .await?;

        let channel_id = response.and_then(|record| record.pub_channel_id.map(from_i64));

        Ok(channel_id)
    }

    ////////////////////////////////////////////////////////////////////////////////////////

    #[instrument]
    pub async fn get_learned(
        &self,
        command_name: &str,
        guild_id: u64,
    ) -> anyhow::Result<Option<String>> {
        let guild_id = to_i64(guild_id);

        let response = sqlx::query!(
            "SELECT content FROM learned_cmd WHERE name = ? AND guild_id = ?",
            command_name,
            guild_id
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(record) = response {
            debug!("command name: {command_name}");
            debug!("command content: {:?}", record.content);
            Ok(record.content)
        } else {
            Ok(None)
        }
    }

    #[instrument]
    pub async fn get_learned_list(&self, guild_id: u64) -> anyhow::Result<Vec<String>> {
        let guild_id = to_i64(guild_id);

        let records = sqlx::query!("SELECT name FROM learned_cmd WHERE guild_id = ?", guild_id)
            .fetch_all(&self.pool)
            .await?;

        let commands = records.iter().map(|record| record.name.clone()).collect();

        Ok(commands)
    }

    pub async fn set_learned(
        &self,
        command_name: &str,
        content: &str,
        guild_id: u64,
    ) -> anyhow::Result<()> {
        let guild_id = to_i64(guild_id);

        sqlx::query!(
            "INSERT INTO learned_cmd (guild_id, name, content) VALUES (?, ?, ?) 
            ON CONFLICT (guild_id, name) DO UPDATE SET content = ?",
            guild_id,
            command_name,
            content,
            content
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    ///////////////////////////////////////////////////////////////////////////////////////
    pub async fn add_roulette_result(
        &self,
        guild_id: u64,
        time_stamp: i64,
        caller_id: u64,
        target_id: u64,
    ) -> anyhow::Result<()> {
        let guild_id = to_i64(guild_id);
        let caller_id = to_i64(caller_id);
        let target_id = to_i64(target_id);

        sqlx::query!(
            "INSERT INTO roulette_count (guild_id, time_stamp, caller_id, target_id)
            values (?, ?, ?, ?)",
            guild_id,
            time_stamp,
            caller_id,
            target_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    #[instrument]
    pub async fn get_roulette_scores(&self, guild_id: u64) -> anyhow::Result<Vec<(u64, u64)>> {
        let guild_id = to_i64(guild_id);

        let records = sqlx::query!(
            "SELECT caller_id, target_id FROM roulette_count WHERE guild_id = ?",
            guild_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(records
            .iter()
            .map(|record| {
                let caller = from_i64(record.caller_id);
                let target = from_i64(record.target_id);
                (caller, target)
            })
            .collect())
    }

    #[instrument]
    pub async fn get_user_roulette_scores(
        &self,
        guild_id: u64,
        user_id: u64,
    ) -> anyhow::Result<Vec<u64>> {
        let guild_id = to_i64(guild_id);
        let user_id = to_i64(user_id);

        let records = sqlx::query!(
            "SELECT target_id FROM roulette_count WHERE guild_id = ? AND caller_id = ?",
            guild_id,
            user_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(records
            .iter()
            .map(|record| from_i64(record.target_id))
            .collect())
    }

    ///////////////////////////////////////////////////////////////////////////////////////

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
