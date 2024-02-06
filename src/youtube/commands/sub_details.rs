use crate::{Context, Error};

#[poise::command(
    slash_command,
    guild_only,
    ephemeral,
    hide_in_help,
    category = "Youtube"
)]
pub(super) async fn sub_details(ctx: Context<'_>, id: String) -> Result<(), Error> {
    let callback = ctx.data().config.brzthook.callback.as_str();
    let topic = format!("https://www.youtube.com/xml/feeds/videos.xml?channel_id={id}");
    let content = format!("https://pubsubhubbub.appspot.com/subscription-details?hub.callback={callback}&hub.topic={topic}&hub.secret=");
    ctx.say(&content).await?;
    Ok(())
}
