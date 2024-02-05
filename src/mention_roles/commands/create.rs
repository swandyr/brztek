use poise::serenity_prelude::{self as serenity, Mentionable};
use tracing::instrument;

use super::queries;
use crate::{Context, Error};

/// Create a new role as a mention role managed by the bot
#[instrument(skip(ctx))]
#[poise::command(slash_command, guild_only, category = "Mention Roles")]
pub async fn create(
    ctx: Context<'_>,
    #[description = "Role name"] name: String,
) -> Result<(), Error> {
    let db = &ctx.data().db;
    let guild_id = ctx.guild_id().ok_or("Not in guild")?;
    let guild_mention_roles = queries::get_role_ids(db, guild_id.get()).await?;

    let guild = ctx.guild().as_deref().cloned().ok_or("Not in guild")?;
    let role = guild
        .create_role(
            ctx,
            serenity::EditRole::new()
                .name(name)
                .permissions(serenity::Permissions::empty()),
        )
        .await?;
    queries::insert(db, guild_id.get(), role.id.get()).await?;

    let content = format!("Mention role {} created", role.mention());
    ctx.reply(content).await?;

    Ok(())
}
