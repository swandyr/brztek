use poise::serenity_prelude::RoleId;

use crate::{Context, Error};

pub async fn roleid_from_name(ctx: Context<'_>, name: &str) -> Result<RoleId, Error> {
    let guild_id = ctx.guild_id().ok_or("Not in guild")?;
    let role_id: Vec<RoleId> = guild_id
        .roles(ctx)
        .await?
        .into_iter()
        .filter(|(_, v)| v.name == name)
        .map(|(k, _)| k)
        .collect();
    assert_eq!(role_id.len(), 1);
    role_id
        .first()
        .copied()
        .ok_or(Error::from("RoleId not found"))
}
