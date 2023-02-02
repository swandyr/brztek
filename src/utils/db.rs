use log::debug;
use serenity::prelude::TypeMapKey;
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::sync::Arc;

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

    pub async fn add_user_xp(&self, user_id: u64) -> anyhow::Result<()> {
        let user_id = to_i64(user_id); // Truncates the value and does not store the real user_id, need fix

        let user_xp = sqlx::query!("SELECT xp FROM edn_ranks WHERE user_id = ?", user_id,)
            .fetch_optional(&self.pool)
            .await?;

        debug!("QUERY RESULT: {user_xp:?}");

        if let Some(record) = user_xp {
            let xp = record.xp.unwrap() + 1;
            sqlx::query!(
                "UPDATE edn_ranks
                SET xp = ?
                WHERE user_id = ?",
                xp,
                user_id,
            )
            .execute(&self.pool)
            .await?;
        } else {
            sqlx::query!(
                "INSERT INTO edn_ranks (user_id, xp) VALUES (?, ?)",
                user_id,
                1,
            )
            .execute(&self.pool)
            .await?;
        };

        Ok(())
    }

    pub async fn get_user_xp(&self, user_id: u64) -> anyhow::Result<u32> {
        let user_id = to_i64(user_id); // Truncates the value and does not store the real user_id, need fix

        let user_xp = sqlx::query!("SELECT xp FROM edn_ranks WHERE user_id = ?", user_id,)
            .fetch_optional(&self.pool)
            .await?;

        let user_xp = if let Some(record) = user_xp {
            record.xp.unwrap_or(0)
        } else {
            0
        };
        Ok(user_xp as u32)
    }

    pub async fn get_all_users_xp(&self) -> anyhow::Result<Vec<(u64, i64, i64)>> {
        let all_users_xp = sqlx::query!("SELECT user_id, xp, level FROM edn_ranks")
            .fetch_all(&self.pool)
            .await
            .unwrap();

        let all_users_xp = all_users_xp
            .iter()
            .map(|c| {
                let user_id = from_i64(c.user_id);
                let xp = c.xp.unwrap_or(0);
                let level = c.level.unwrap_or(0);
                (user_id, xp, level)
            })
            .collect();

        Ok(all_users_xp)
    }

    #[allow(dead_code)]
    pub async fn update_levels(&self) -> anyhow::Result<()> {
        todo!()
    }

    pub async fn delete_table(&self) -> anyhow::Result<()> {
        sqlx::query!("DELETE FROM edn_ranks")
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}

fn to_i64(unsigned: u64) -> i64 {
    let bit_cast = unsigned.to_be_bytes();
    i64::from_be_bytes(bit_cast)
}

fn from_i64(signed: i64) -> u64 {
    let bit_cast = signed.to_be_bytes();
    u64::from_be_bytes(bit_cast)
}
