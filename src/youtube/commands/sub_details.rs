use poise::serenity_prelude::futures::{self, Stream, StreamExt};

use super::queries;
use crate::{Context, Error};

async fn autocomplete<'a>(ctx: Context<'_>, partial: &'a str) -> impl Stream<Item = String> + 'a {
    let db = &ctx.data().db;
    let subs_list = queries::get_subs_list(db).await.unwrap();
    futures::stream::iter(subs_list).map(|sub| sub.yt_channel_name)
}

#[poise::command(
    slash_command,
    guild_only,
    ephemeral,
    hide_in_help,
    category = "Youtube"
)]
pub(super) async fn sub_details(
    ctx: Context<'_>,
    #[autocomplete = "autocomplete"] name: String,
) -> Result<(), Error> {
    let db = &ctx.data().db;
    let guild_id = ctx.guild_id().ok_or("Not in guild")?;
    let id = queries::get_sub(db, &name, guild_id.get())
        .await?
        .ok_or("Youtube Id not found")?
        .yt_channel_id;
    let callback = ctx.data().config.brzthook.callback.as_str();
    let topic = format!("https://www.youtube.com/xml/feeds/videos.xml?channel_id={id}");
    let content = format!("https://pubsubhubbub.appspot.com/subscription-details?hub.callback={callback}&hub.topic={topic}&hub.secret=");
    ctx.say(&content).await?;
    Ok(())
}
