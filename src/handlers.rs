use std::time::Instant;

use poise::serenity_prelude::{
    self as serenity,
    audit_log::{Action, MemberAction},
    CreateMessage, EditMessage, GuildId, Member, Mentionable, Message, User,
};
use rand::{thread_rng, Rng};
use tracing::{debug, info, instrument, log::warn, trace};

use crate::{clearurl::clear_url, levels::func::message_xp, database};
use crate::{Context, Data, Error};

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
    message_xp::add_xp(ctx, user_data, &guild_id, &channel_id, &user_id).await?;
    debug!("add_xp finished in {} µs", t_0.elapsed().as_micros());

    Ok(())
}

#[instrument(skip(ctx))]
pub async fn member_addition_handler(
    new_member: &Member,
    ctx: &serenity::Context,
) -> Result<(), Error> {
    let mention = new_member.mention();
    let content = format!("A wild **{mention}** appeared !");

    let system_channel_id = new_member
        .guild_id
        .to_guild_cached(ctx)
        .unwrap()
        .system_channel_id
        .unwrap();
    system_channel_id
        .send_message(&ctx.http, CreateMessage::new().content(content))
        .await?;

    Ok(())
}

#[instrument(skip(ctx))]
pub async fn member_removal_handler(
    guild_id: &GuildId,
    user: &User,
    ctx: &serenity::Context,
) -> Result<(), Error> {
    let username = user.name.to_string();
    //let mut content = format!("RIP **{username}**, you'll be missed");
    let mut content = format!("✝️ RIP en paix **{username}** , un 👼 parti trop tôt 🕯️");

    let system_channel_id = guild_id
        .to_guild_cached(ctx)
        .unwrap()
        .system_channel_id
        .unwrap();

    // if bot can read audit logs
    if guild_id
        .to_guild_cached(ctx)
        .unwrap()
        .role_by_name("brztek")
        .unwrap()
        .has_permission(serenity::Permissions::VIEW_AUDIT_LOG)
    {
        info!("Checking audit_logs");
        let audit_logs = guild_id
            .audit_logs(&ctx.http, None, None, None, Some(1))
            .await
            .unwrap();
        let last_log = audit_logs.entries.first().unwrap();

        // if last action is the kick of the user, change message content accordingly
        if matches!(last_log.action, Action::Member(MemberAction::Kick)) {
            if let Some(target_id) = last_log.target_id {
                if target_id == user.id.get() {
                    content = format!("**{username}** has got his ass out of here!");
                }
            }
        }
    } else {
        warn!("Bot is missing permission VIEW_AUDIT_LOG");
    }

    system_channel_id
        .send_message(&ctx.http, CreateMessage::new().content(content))
        .await?;

    Ok(())
}

///////////////////////////////////////////////////////////////
