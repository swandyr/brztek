use poise::serenity_prelude::{
    self as serenity,
    futures::{self, Stream, StreamExt},
    Role, RoleId,
};
use tracing::instrument;

use super::queries;
use crate::{Context, Error};

async fn autocomplete<'a>(ctx: Context<'_>, partial: &'a str) -> impl Stream<Item = String> + 'a {
    futures::stream::iter(
        ctx.guild_id()
            .unwrap()
            .roles(ctx)
            .await
            .unwrap()
            .into_values(),
    )
    .map(|r| r.name)
}

/// Add an existing role to the mention roles managed by the bot
#[instrument(skip(ctx))]
#[poise::command(slash_command, guild_only, ephemeral, category = "Mention Roles")]
pub async fn add(
    ctx: Context<'_>,
    #[description = "Role to add as mention role"] role: Role,
) -> Result<(), Error> {
    let db = &ctx.data().db;
    let guild_id = ctx.guild_id().ok_or("Not in guild")?;
    let role_id = role.id;

    queries::insert(db, guild_id.get(), role_id.get()).await?;
    ctx.reply("Done").await?;

    Ok(())
}
