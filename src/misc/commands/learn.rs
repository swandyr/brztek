use tracing::instrument;

use super::queries;
use crate::{Context, Error};

/// Make the bot remember.
///
/// Save a named command with a link the bot will post when responding
/// to the command.
#[instrument(skip(ctx))]
#[poise::command(prefix_command, slash_command, guild_only, category = "Misc")]
pub async fn learn(
    ctx: Context<'_>,
    #[description = "Name"] name: String,
    #[description = "Link"] link: String,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Not in guild")?.get();
    let db = &ctx.data().db;

    queries::set_learned(db, &name, &link, guild_id).await?;

    ctx.say(format!("I know {name}")).await?;

    Ok(())
}
