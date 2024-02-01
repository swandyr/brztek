use brzthook::Mode;
use crate::{Context, Error, youtube::queries};

/// Unsub and delete a webhook
///
/// Input the exact name of the channel (use "/yt list" if needed)
#[allow(unused)]
#[poise::command(
slash_command,
guild_only,
required_permissions = "MANAGE_WEBHOOKS",
ephemeral,
category = "Youtube"
)]
pub(super) async fn unsub(
    ctx: Context<'_>,
    #[description = "Name of the channel"] name: String,
) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;

    let db = &ctx.data().db;
    let guild_id = ctx.guild_id().ok_or("Not in guild")?;
    let Some(sub) = queries::get_sub(db, &name, guild_id.get()).await? else {
        ctx.say("No channel found with this name.").await?;
        return Ok(());
    };

    let author_id = sub.yt_channel_id;

    ctx.data()
        .hook_listener
        .subscribe(&author_id, Mode::Unsubscribe)?;

    queries::delete_sub(db, &author_id, ctx.guild_id().unwrap().get()).await?;

    let content = format!("Unsubbed to {name}");
    ctx.say(&content).await?;
    Ok(())
}