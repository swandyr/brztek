use poise::serenity_prelude as serenity;
use tracing::info;

use crate::levels::xp;
use crate::Data;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[allow(dead_code)]
async fn is_admin(ctx: Context<'_>, member: serenity::PartialMember) -> Result<bool, Error> {
    Ok(member.roles.iter().any(|r| {
        r.to_role_cached(ctx).map_or(false, |r| {
            r.has_permission(serenity::Permissions::ADMINISTRATOR)
        })
    }))
}

#[poise::command(
    prefix_command,
    slash_command,
    subcommands("set_pub", "set_user", "spam_delay", "min_xp_gain", "max_xp_gain"),
    required_permissions = "ADMINISTRATOR",
    category = "Admin"
)]
pub async fn admin(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(prefix_command, slash_command, category = "Admin")]
pub async fn set_pub(
    ctx: Context<'_>,
    #[description = "The channel to set to public"] channel: serenity::ChannelId,
) -> Result<(), Error> {
    let guild_id = if let Some(id) = ctx.guild_id() {
        id.0
    } else {
        ctx.say("Must be in guild").await?;
        return Ok(());
    };
    let channel_id = channel.0;

    ctx.data()
        .db
        .set_pub_channel_id(channel_id, guild_id)
        .await?;

    info!("Channel {channel_id} set to pub for guild {guild_id}");

    Ok(())
}

#[poise::command(prefix_command, slash_command, category = "Admin")]
pub async fn set_user(
    ctx: Context<'_>,
    #[description = "User to modify"] user: serenity::UserId,
    #[description = "Messages count"] messages: i64,
    #[description = "Amount of Xp"] xp: i64,
) -> Result<(), Error> {
    let guild_id = if let Some(id) = ctx.guild_id() {
        id.0
    } else {
        ctx.say("Must be in guild").await?;
        return Ok(());
    };
    let user_id = user.0;

    let level = xp::calculate_level_from_xp(xp);

    let mut user_level = ctx.data().db.get_user(user_id, guild_id).await?;
    user_level.xp = xp;
    user_level.level = level;
    user_level.messages = messages;
    ctx.data().db.update_user(&user_level, guild_id).await?;

    info!("Admin updated user {user_id} in guild {guild_id}: {xp} - {level} - {messages}");

    let username = user.to_user(ctx).await?;
    ctx.send(|b| {
        let content = format!("{} is now level {}", username, user_level.level);
        b.content(content)
    })
    .await?;

    Ok(())
}

#[poise::command(prefix_command, slash_command, category = "Admin")]
pub async fn spam_delay(
    ctx: Context<'_>,
    #[description = "Delay in seconds. Leave empty to get the actual value."] value: Option<i64>,
) -> Result<(), Error> {
    let guild_id = if let Some(id) = ctx.guild_id() {
        id.0
    } else {
        ctx.say("Must be in guild").await?;
        return Ok(());
    };

    if let Some(value) = value {
        ctx.data().db.set_spam_delay(guild_id, value).await?;
    }

    let value = ctx.data().db.get_spam_delay(guild_id).await?;
    ctx.send(|b| {
        let content = format!("Spam delay is set to {value} seconds.");
        b.content(content)
    })
    .await?;

    Ok(())
}

#[poise::command(prefix_command, slash_command, category = "Admin")]
pub async fn min_xp_gain(
    ctx: Context<'_>,
    #[description = "Min xp points thaht can be gained. Leave empty to get the actual value."]
    value: Option<i64>,
) -> Result<(), Error> {
    let guild_id = if let Some(id) = ctx.guild_id() {
        id.0
    } else {
        ctx.say("Must be in guild").await?;
        return Ok(());
    };

    if let Some(value) = value {
        ctx.data().db.set_min_xp_gain(guild_id, value).await?;
    }

    let value = ctx.data().db.get_min_xp_gain(guild_id).await?;
    ctx.send(|b| {
        let content = format!("Min Xp gain is set to {value} points.");
        b.content(content)
    })
    .await?;

    Ok(())
}

#[poise::command(prefix_command, slash_command, category = "Admin")]
pub async fn max_xp_gain(
    ctx: Context<'_>,
    #[description = "Maximum xp points that can be gained. Leave empty to get the actual value."]
    value: Option<i64>,
) -> Result<(), Error> {
    let guild_id = if let Some(id) = ctx.guild_id() {
        id.0
    } else {
        ctx.say("Must be in guild").await?;
        return Ok(());
    };

    if let Some(value) = value {
        ctx.data().db.set_max_xp_gain(guild_id, value).await?;
    }
    let value = ctx.data().db.get_max_xp_gain(guild_id).await?;
    ctx.send(|b| {
        let content = format!("Max Xp gain is set to {value} points.");
        b.content(content)
    })
    .await?;

    Ok(())
}
