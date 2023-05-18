use tracing::instrument;

use crate::db::{from_i64, to_i64, Db};

#[instrument]
pub async fn add_roulette_result(
    db: &Db,
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
    .execute(&db.pool)
    .await?;

    Ok(())
}

#[instrument]
pub async fn get_roulette_scores(db: &Db, guild_id: u64) -> anyhow::Result<Vec<(u64, u64)>> {
    let guild_id = to_i64(guild_id);

    let records = sqlx::query!(
        "SELECT caller_id, target_id FROM roulette_count WHERE guild_id = ?",
        guild_id
    )
    .fetch_all(&db.pool)
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
