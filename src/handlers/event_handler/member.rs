use poise::serenity_prelude::{
    self as serenity,
    audit_log::{Action, MemberAction},
    CreateMessage, GuildId, Member, Mentionable, User,
};
use tracing::{info, instrument, trace, warn};

use crate::Error;

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
