use tracing::{info, instrument};

use crate::{Context, Error};

#[instrument(skip(ctx))]
#[poise::command(slash_command, owners_only, hide_in_help, ephemeral)]
pub async fn shutdown(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("Shutting down...").await?;
    info!("Closing database...");
    ctx.data().db.close().await;
    info!("Shutting down all shards...");
    ctx.framework().shard_manager().shutdown_all().await;
    Ok(())
}
