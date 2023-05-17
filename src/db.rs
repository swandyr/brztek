use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use tracing::instrument;

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
    pub async fn run_migrations(&self) -> anyhow::Result<()> {
        sqlx::migrate!("./migrations").run(&self.pool).await?;
        Ok(())
    }

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
