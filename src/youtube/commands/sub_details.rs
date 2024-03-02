use piet_common::TextStorage;
use poise::serenity_prelude::{CacheHttp, ChannelId, Mentionable};

use super::{func::autocomplete_sublist, queries};
use crate::{Context, Error};

#[poise::command(
    slash_command,
    guild_only,
    ephemeral,
    hide_in_help,
    category = "Youtube"
)]
pub(super) async fn sub_details(
    ctx: Context<'_>,
    #[autocomplete = "autocomplete_sublist"] name: String,
) -> Result<(), Error> {
    let db = &ctx.data().db;
    let guild_id = ctx.guild_id().ok_or("Not in guild")?;
    let sub = queries::get_sub(db, &name, guild_id.get())
        .await?
        .ok_or("Youtube Id not found")?;
    let callback = ctx.data().config.brzthook.callback.as_str();
    let topic = format!(
        "https://www.youtube.com/xml/feeds/videos.xml?channel_id={}",
        &sub.yt_channel_id
    );
    let pshb_link = format!("https://pubsubhubbub.appspot.com/subscription-details?hub.callback={callback}&hub.topic={topic}&hub.secret=");
    let post_chan = ChannelId::new(sub.post_channel_id);

    let content = format!(
        r#"# Sub details
- Channel Name: {}
- Youtube Id: {},
- Discord Channel: {}
- Expire on: {}
- [PubSubHubHub link]({})"#,
        &sub.yt_channel_name,
        &sub.yt_channel_id,
        post_chan.mention(),
        sub.expire_on,
        pshb_link
    );
    ctx.say(&content).await?;
    Ok(())
}
