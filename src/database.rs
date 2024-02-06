use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use tracing::instrument;

use crate::Error;

#[derive(Debug)]
pub struct Db {
    pub pool: SqlitePool,
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
    pub async fn run_migrations(&self) -> Result<(), Error> {
        sqlx::migrate!("./migrations").run(&self.pool).await?;
        Ok(())
    }

    #[instrument]
    pub async fn close(&self) {
        self.pool.close().await;
    }
}

/// Bit-cast u64 (user.id in Discord API) to i64 (stored in the `SQLite` database).
pub const fn to_i64(unsigned: u64) -> i64 {
    let bit_cast = unsigned.to_be_bytes();
    i64::from_be_bytes(bit_cast)
}

/// Bit-cast i64 (stored in `SQLite` database) to u64 (user.id in Discord API).
pub const fn from_i64(signed: i64) -> u64 {
    let bit_cast = signed.to_be_bytes();
    u64::from_be_bytes(bit_cast)
}

#[instrument]
pub async fn add_guild(db: &Db, guild_id: u64) -> Result<(), Error> {
    let guild_id = to_i64(guild_id);

    sqlx::query!("INSERT OR IGNORE INTO guilds (id) VALUES (?)", guild_id)
        .execute(&db.pool)
        .await?;

    Ok(())
}

#[instrument]
pub async fn add_user(db: &Db, user_id: u64) -> Result<(), Error> {
    let user_id = to_i64(user_id);

    sqlx::query!("INSERT OR IGNORE INTO users (id) VALUES (?)", user_id)
        .execute(&db.pool)
        .await?;

    Ok(())
}

#[instrument]
pub async fn increment_cmd(db: &Db, cmd: &str, guild_id: u64) -> Result<(), Error> {
    let guild_id = to_i64(guild_id);

    sqlx::query!(
        "INSERT OR IGNORE INTO cmd_count (guild_id, command) VALUES (?, ?);
    UPDATE cmd_count SET count = count + 1 WHERE guild_id = ? AND command = ?",
        guild_id,
        cmd,
        guild_id,
        cmd
    )
    .execute(&db.pool)
    .await?;

    Ok(())
}
