use poise::serenity_prelude::UserId;
use tracing::instrument;

use super::user_level::UserLevel;
use crate::db::{from_i64, to_i64, Db};
use crate::Error;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
struct UserSql {
    user_id: i64,
    guild_id: i64,
    xp: i64,
    level: i64,
    rank: i64,
    last_message: i64,
}

impl From<UserSql> for UserLevel {
    fn from(value: UserSql) -> Self {
        Self {
            user_id: UserId::from(from_i64(value.user_id)),
            xp: value.xp,
            level: value.level,
            rank: value.rank,
            last_message: value.last_message,
        }
    }
}

/// Return `UserLevel` corresponding to `user_id` in the database.
///
/// If no user is found, create a new entry with `user_id` and returns
/// new `UserLevel`.
#[instrument]
pub async fn get_user(db: &Db, user_id: u64, guild_id: u64) -> Result<UserLevel, Error> {
    // Bit-cast `user_id` from u64 to i64, as SQLite does not support u64 integer
    let user_id = to_i64(user_id);
    let guild_id = to_i64(guild_id);

    let response = sqlx::query_as!(
        UserSql,
        "SELECT * FROM levels WHERE user_id = ? AND guild_id = ?",
        user_id,
        guild_id
    )
    .fetch_optional(&db.pool)
    .await?;

    if let Some(record) = response {
        Ok(UserLevel::from(record))
    } else {
        sqlx::query!(
            "INSERT INTO levels (user_id, guild_id) VALUES (?, ?)",
            user_id,
            guild_id
        )
        .execute(&db.pool)
        .await?;
        Ok(UserLevel::new(from_i64(user_id)))
    }
}

/// Update user's entry in the database with new values.
#[instrument]
pub async fn update_user(db: &Db, user: &UserLevel, guild_id: u64) -> Result<(), Error> {
    // Bit-cast `user_id` from u64 to i64, as SQLite does not support u64 integer
    let user_id = to_i64(user.user_id.get());
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
    .execute(&db.pool)
    .await?;

    Ok(())
}

/// Update user rank in the database
#[instrument]
pub async fn update_ranks(db: &Db, users: Vec<UserLevel>, guild_id: u64) -> Result<(), Error> {
    let guild_id = to_i64(guild_id);

    for user in users {
        let user_id = to_i64(user.user_id.get());

        sqlx::query!(
            "UPDATE levels
                SET rank = ?
                WHERE user_id = ? AND guild_id = ?",
            user.rank,
            user_id,
            guild_id
        )
        .execute(&db.pool)
        .await?;
    }

    Ok(())
}

/// Get all entries in the database and returns a `Vec<UserLevel>`
#[instrument]
pub async fn get_all_users(db: &Db, guild_id: u64) -> Result<Vec<UserLevel>, Error> {
    let guild_id = to_i64(guild_id);

    let response = sqlx::query_as!(UserSql, "SELECT * FROM levels WHERE guild_id = ?", guild_id)
        .fetch_all(&db.pool)
        .await?;

    let all_users = response
        .iter()
        .map(|record| UserLevel::from(*record))
        .collect();

    Ok(all_users)
}

/// Import levels from Mee6.
///
/// Clear all users entries corresponding to the guild_id first,
/// and insert all new entries in hte `users: Vec<UserLevel>
#[instrument]
pub async fn import_from_mee6(db: &Db, users: Vec<UserLevel>, guild_id: u64) -> Result<(), Error> {
    let guild_id = to_i64(guild_id);

    sqlx::query!("DELETE FROM levels WHERE guild_id = ?", guild_id)
        .execute(&db.pool)
        .await?;

    for user in users {
        let user_id = to_i64(user.user_id.get());
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
        .execute(&db.pool)
        .await?;
    }

    Ok(())
}
