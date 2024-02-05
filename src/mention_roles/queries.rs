use tracing::instrument;

use crate::{
    database::{from_i64, to_i64, Db},
    Error,
};

#[instrument]
pub async fn get_role_ids(db: &Db, guild_id: u64) -> Result<Vec<u64>, Error> {
    let guild_id = to_i64(guild_id);

    let records = sqlx::query!(
        "SELECT role_id FROM mention_roles WHERE guild_id=?",
        guild_id
    )
    .fetch_all(&db.pool)
    .await?;

    let role_ids = records.iter().map(|r| from_i64(r.role_id)).collect();

    Ok(role_ids)
}

#[instrument]
pub async fn insert(db: &Db, guild_id: u64, role_id: u64) -> Result<(), Error> {
    let guild_id = to_i64(guild_id);
    let role_id = to_i64(role_id);

    sqlx::query!(
        "INSERT INTO mention_roles(guild_id, role_id) VALUES (?, ?)",
        guild_id,
        role_id,
    )
    .execute(&db.pool)
    .await?;

    Ok(())
}

#[instrument]
pub async fn delete(db: &Db, guild_id: u64, role_id: u64) -> Result<(), Error> {
    let guild_id = to_i64(guild_id);
    let role_id = to_i64(role_id);

    sqlx::query!(
        "DELETE FROM mention_roles WHERE guild_id = ? AND role_id = ?",
        guild_id,
        role_id
    )
    .execute(&db.pool)
    .await?;

    Ok(())
}
