use tracing::{debug, instrument};

use crate::db::{from_i64, to_i64, Db};

////////////////////////////////////////////////////////////////////////////////////////

#[instrument]
pub async fn set_role_color(
    db: &Db,
    guild_id: u64,
    user_id: u64,
    role_id: u64,
) -> anyhow::Result<()> {
    let guild_id = to_i64(guild_id);
    let user_id = to_i64(user_id);
    let role_id = to_i64(role_id);

    sqlx::query!(
        "INSERT INTO members (guild_id, user_id, role_color_id) VALUES (?, ?, ?)
            ON CONFLICT (guild_id, user_id) DO UPDATE SET role_color_id = ?",
        guild_id,
        user_id,
        role_id,
        role_id
    )
    .execute(&db.pool)
    .await?;

    Ok(())
}

#[instrument]
pub async fn get_role_color(db: &Db, guild_id: u64, user_id: u64) -> anyhow::Result<Option<u64>> {
    let guild_id = to_i64(guild_id);
    let user_id = to_i64(user_id);

    let response = sqlx::query!(
        "SELECT role_color_id FROM members WHERE guild_id = ? AND user_id = ?",
        guild_id,
        user_id
    )
    .fetch_optional(&db.pool)
    .await?;

    let role_id = response.and_then(|record| record.role_color_id.map(from_i64));

    Ok(role_id)
}

///////////////////////////////////////////////////////////////////////////////////////:

#[instrument]
pub async fn get_learned(
    db: &Db,
    command_name: &str,
    guild_id: u64,
) -> anyhow::Result<Option<String>> {
    let guild_id = to_i64(guild_id);

    let response = sqlx::query!(
        "SELECT content FROM learned_cmds WHERE name = ? AND guild_id = ?",
        command_name,
        guild_id
    )
    .fetch_optional(&db.pool)
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
pub async fn get_learned_list(db: &Db, guild_id: u64) -> anyhow::Result<Vec<String>> {
    let guild_id = to_i64(guild_id);

    let records = sqlx::query!("SELECT name FROM learned_cmds WHERE guild_id = ?", guild_id)
        .fetch_all(&db.pool)
        .await?;

    let commands = records.iter().map(|record| record.name.clone()).collect();

    Ok(commands)
}

pub async fn set_learned(
    db: &Db,
    command_name: &str,
    content: &str,
    guild_id: u64,
) -> anyhow::Result<()> {
    let guild_id = to_i64(guild_id);

    sqlx::query!(
        "INSERT INTO learned_cmds (guild_id, name, content) VALUES (?, ?, ?) 
            ON CONFLICT (guild_id, name) DO UPDATE SET content = ?",
        guild_id,
        command_name,
        content,
        content
    )
    .execute(&db.pool)
    .await?;

    Ok(())
}
