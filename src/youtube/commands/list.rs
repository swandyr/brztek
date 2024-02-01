use crate::{Context, Error, youtube::queries};

/// List all subs in the guild
#[poise::command(slash_command, guild_only, category = "Youtube")]
pub(super) async fn list(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Not in guild")?;
    let subs: Vec<String> = queries::get_subs_list(&ctx.data().db)
        .await?
        .into_iter()
        .filter(|s| s.guild_id == guild_id.get())
        .map(|s| s.yt_channel_name)
        .collect();

    let content = format!("**List of subscribed channels:**\n>>> {}", subs.join("\n"));
    ctx.say(&content).await?;

    Ok(())
}