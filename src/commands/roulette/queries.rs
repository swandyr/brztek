use poise::serenity_prelude::UserId;
use tracing::instrument;

use super::Roulette;
use crate::db::{from_i64, to_i64, Db};

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
struct RouletteSql {
    timestamp: i64,
    caller_id: i64,
    target_id: i64,
    rff_triggered: Option<u8>,
}

impl From<RouletteSql> for Roulette {
    fn from(value: RouletteSql) -> Self {
        Self {
            timestamp: value.timestamp,
            caller_id: UserId::from(from_i64(value.caller_id)),
            target_id: UserId::from(from_i64(value.target_id)),
            rff_triggered: value.rff_triggered,
        }
    }
}

#[instrument]
pub async fn add_roulette_result(db: &Db, guild_id: u64, roulette: Roulette) -> anyhow::Result<()> {
    let guild_id = to_i64(guild_id);
    let timestamp = roulette.timestamp;
    let caller_id = to_i64(roulette.caller_id.0);
    let target_id = to_i64(roulette.target_id.0);
    let rff_triggered = roulette.rff_triggered;

    sqlx::query!(
        "INSERT INTO roulettes(guild_id, timestamp, caller_id, target_id, rff_triggered)
                VALUES (?, ?, ?, ?, ?)",
        guild_id,
        timestamp,
        caller_id,
        target_id,
        rff_triggered,
    )
    .execute(&db.pool)
    .await?;

    Ok(())
}

#[instrument]
pub async fn get_roulette_scores(db: &Db, guild_id: u64) -> anyhow::Result<Vec<Roulette>> {
    let guild_id = to_i64(guild_id);

    let records = sqlx::query_as!(
        RouletteSql,
        r#"SELECT 
            timestamp, 
            caller_id,
            target_id, 
            rff_triggered as "rff_triggered: u8"
        FROM roulettes WHERE guild_id = ?"#,
        guild_id
    )
    .fetch_all(&db.pool)
    .await?;

    Ok(records.into_iter().map(Roulette::from).collect())
}
