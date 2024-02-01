use tracing::instrument;

use crate::{Context, Error};

/// Ping the bot!
///
/// He'll pong you back.
#[instrument(skip(ctx))]
#[poise::command(prefix_command, slash_command, category = "Misc")]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("Pong!").await?;
    Ok(())
}
