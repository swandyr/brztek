use poise::serenity_prelude::UserId;
use tracing::instrument;

use super::models::{Roulette, RouletteSql};
use crate::{
    database::{from_i64, to_i64, Db},
    Error,
};

#[instrument]
pub async fn add_roulette_result(db: &Db, guild_id: u64, roulette: Roulette) -> Result<(), Error> {
    let guild_id = to_i64(guild_id);
    let timestamp = roulette.timestamp;
    let caller_id = to_i64(roulette.caller_id.get());
    let target_id = to_i64(roulette.target_id.get());
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
pub async fn get_roulette_scores(db: &Db, guild_id: u64) -> Result<Vec<Roulette>, Error> {
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
