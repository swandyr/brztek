use poise::serenity_prelude::{
    self as serenity,
    futures::{self, Stream, StreamExt},
    Role,
};
use tracing::instrument;

use super::queries;
use crate::{Context, Error};

//TODO: Make autocomplete works

async fn autocomplete<'a>(ctx: Context<'_>, partial: &'a str) -> impl Stream<Item = Role> + 'a {
    let db = &ctx.data().db;
    let guild_id = ctx.guild_id().ok_or("Not in guild").unwrap();
    let mention_role_ids = queries::get_role_ids(db, guild_id.get()).await.unwrap();
    let mention_roles: Vec<Role> = mention_role_ids
        .into_iter()
        .map(|id| serenity::RoleId::from(id).to_role_cached(ctx).unwrap())
        .collect();
    dbg!(&mention_roles);
    futures::stream::iter(mention_roles)
}

async fn autocomplete_dbg<'a>(
    ctx: Context<'_>,
    partial: &'a str,
) -> impl Stream<Item = String> + 'a {
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

#[instrument(skip(ctx))]
#[poise::command(slash_command, guild_only, category = "Mention Roles")]
pub async fn delete(
    ctx: Context<'_>,
    #[description = "Role name"]
    #[autocomplete = "autocomplete_dbg"]
    name: String,
) -> Result<(), Error> {
    let content = format!("Deleting role {}", name);
    ctx.reply(content).await?;
    Ok(())
}
