use poise::serenity_prelude::{
    self as serenity,
    futures::{self, Stream, StreamExt},
    Role, RoleId,
};
use tracing::instrument;

use super::{queries, util};
use crate::{Context, Error};

async fn autocomplete<'a>(ctx: Context<'_>, partial: &'a str) -> impl Stream<Item = String> + 'a {
    let db = &ctx.data().db;
    let guild_id = ctx.guild_id().ok_or("Not in guild").unwrap();
    let mention_role_ids = queries::get_role_ids(db, guild_id.get()).await.unwrap();
    let mention_role_ids: Vec<RoleId> = mention_role_ids
        .into_iter()
        .map(serenity::RoleId::from)
        .collect();
    let mention_roles: Vec<String> = ctx
        .guild_id()
        .unwrap()
        .roles(ctx)
        .await
        .unwrap()
        .into_iter()
        .filter(|(k, _)| mention_role_ids.contains(k))
        .map(|(_, v)| v.name)
        .collect();
    futures::stream::iter(mention_roles)
}

/// Delete a mention role from the bot and discord
#[instrument(skip(ctx))]
#[poise::command(slash_command, guild_only, ephemeral, category = "Mention Roles")]
pub async fn delete(
    ctx: Context<'_>,
    #[description = "Role name"]
    #[autocomplete = "autocomplete"]
    name: String,
) -> Result<(), Error> {
    let db = &ctx.data().db;
    let guild_id = ctx.guild_id().ok_or("Not in guild")?;
    let role_id = util::roleid_from_name(ctx, &name).await?;
    guild_id.delete_role(ctx, role_id).await?;
    queries::delete(db, guild_id.get(), role_id.get()).await?;

    let content = format!("Deleted role {}", name);
    ctx.reply(content).await?;
    Ok(())
}
