use std::time::Instant;

use poise::serenity_prelude::{self as serenity, EditMessage, Message};
use tracing::{debug, info, instrument, trace};

use crate::{clearurl::clear_url, database, levels, Data, Error};

#[instrument(skip_all, fields(guild=new_message.guild_id.unwrap().name(ctx), author=new_message.author.name))]
pub async fn message_handler(
    new_message: &Message,
    ctx: &serenity::Context,
    user_data: &Data,
) -> Result<(), Error> {
    trace!(
        "Handling new message in guild: {:?}",
        new_message.guild_id.unwrap().name(ctx).unwrap()
    );

    let user_id = new_message.author.id;
    let channel_id = new_message.channel_id;
    let guild_id = new_message.guild_id.unwrap();

    // Split the message content on whitespace and new line char
    let content = new_message.content.split(&[' ', '\n']);
    // Filter on any links contained in the message content
    let links = content
        .filter(|f| f.starts_with("https://") || f.starts_with("http://"))
        .collect::<Vec<&str>>();
    for link in links {
        info!("Cleaning link {}", link);
        let t_0 = Instant::now();
        if let Some(cleaned) = clear_url(link).await? {
            info!("Cleaned link -> {}", cleaned);
            // Send message with cleaned url
            let content = format!("Cleaned that shit for you\n{cleaned}");
            channel_id.say(ctx, content).await?;

            // Delete embeds in original message
            channel_id
                .message(ctx, new_message.id)
                .await?
                // ctx cache return NotAuthor error, but ctx.http works fine
                .edit(&ctx.http, EditMessage::new().suppress_embeds(true))
                .await?;
        }
        debug!("clear_url finished in {} µs", t_0.elapsed().as_micros());
    }

    // User gains xp on message
    let t_0 = Instant::now();
    let db = &user_data.db;
    database::add_user(db, user_id.get()).await?;
    levels::func::message_xp::add_xp(ctx, user_data, &guild_id, &channel_id, &user_id).await?;
    debug!("add_xp finished in {} µs", t_0.elapsed().as_micros());

    Ok(())
}
