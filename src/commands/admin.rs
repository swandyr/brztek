use poise::serenity_prelude::{self as serenity, Mentionable};
use tracing::{debug, info, instrument};

use crate::levels::user_level::UserLevel;
use crate::levels::xp;
use crate::Data;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

/// Admin commands
///
/// Prefix subcommands that need Administrator priviledges.
///
/// Available subcommands are set_pub, set_user, spam_delay, min_xp_gain, max_xp_gain.
#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    subcommands("set_pub", "set_user", "spam_delay", "min_xp_gain", "max_xp_gain"),
    required_permissions = "ADMINISTRATOR",
    category = "Admin"
)]
pub async fn admin(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

async fn autocomplete_channel<'a>(
    ctx: Context<'_>,
    _partial: &'a str,
) -> impl Iterator<Item = serenity::GuildChannel> {
    ctx.guild()
        .unwrap()
        .channels(ctx)
        .await
        .unwrap()
        .into_values()
        .filter(|chan| chan.is_text_based()) //? filter doesn't seem to work
}

/// Set a channel public
#[instrument(skip(ctx))]
#[poise::command(prefix_command, slash_command, guild_only, category = "Admin")]
async fn set_pub(
    ctx: Context<'_>,
    #[description = "The channel to set to public"]
    #[autocomplete = "autocomplete_channel"]
    channel: serenity::GuildChannel,
) -> Result<(), Error> {
    let Some(guild_id) = ctx.guild_id() else {
        ctx.say("Must be in guild").await?;
        return Ok(());
    };
    let channel_id = channel.id;

    ctx.data()
        .db
        .set_pub_channel_id(channel_id.0, guild_id.0)
        .await?;

    info!("Channel {channel_id} set to pub for guild {guild_id}");

    ctx.say(format!("{} is the new pub", channel.name()))
        .await?;

    Ok(())
}

/// Set the user's xp points
#[instrument(skip(ctx))]
#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    ephemeral,
    category = "Admin"
)]
async fn set_user(
    ctx: Context<'_>,
    #[description = "User to modify"] user: serenity::User,
    #[description = "Amount of Xp"]
    #[min = 0]
    xp: u32,
) -> Result<(), Error> {
    let Some(guild_id) = ctx.guild_id() else {
        ctx.say("Must be in guild").await?;
        return Ok(());
    };
    let user_id = user.id;

    let level = xp::calculate_level_from_xp(xp as i64);

    let mut user_level = ctx.data().db.get_user(user_id.0, guild_id.0).await?;
    user_level.xp = xp as i64;
    user_level.level = level;
    ctx.data().db.update_user(&user_level, guild_id.0).await?;

    info!("Admin updated user {user_id} in guild {guild_id}: {xp} - {level}");

    ctx.say(format!("{} is now level {}", user.name, user_level.level))
        .await?;

    Ok(())
}

/// Specifie the spam delay
///
/// A user will not gain xp if his last message was sent earlier than the spam delay
#[instrument(skip(ctx))]
#[poise::command(prefix_command, slash_command, guild_only, category = "Admin")]
async fn spam_delay(
    ctx: Context<'_>,
    #[description = "Delay in seconds. Leave empty to get the actual value."]
    #[min = 0]
    value: Option<u32>,
) -> Result<(), Error> {
    println!("SPAMDELAY: {value:?}");
    let Some(guild_id) = ctx.guild_id() else {
        ctx.say("Must be in guild").await?;
        return Ok(());
    };

    if let Some(value) = value {
        ctx.data()
            .db
            .set_spam_delay(guild_id.0, value as i64)
            .await?;
    }

    let value = ctx.data().db.get_spam_delay(guild_id.0).await?;
    ctx.say(format!("Spam delay is set to {value} seconds."))
        .await?;

    Ok(())
}

/// Set the minimum xp gain per message
#[instrument(skip(ctx))]
#[poise::command(prefix_command, slash_command, guild_only, category = "Admin")]
async fn min_xp_gain(
    ctx: Context<'_>,
    #[description = "Min xp points thaht can be gained. Leave empty to get the actual value."]
    #[min = 0]
    value: Option<u32>,
) -> Result<(), Error> {
    let Some(guild_id) = ctx.guild_id() else {
        ctx.say("Must be in guild").await?;
        return Ok(());
    };

    if let Some(value) = value {
        ctx.data()
            .db
            .set_min_xp_gain(guild_id.0, value as i64)
            .await?;
    }

    let value = ctx.data().db.get_min_xp_gain(guild_id.0).await?;
    ctx.say(format!("Min Xp gain is set to {value} points."))
        .await?;

    Ok(())
}

/// Set the maximum xp gain per message
#[instrument(skip(ctx))]
#[poise::command(prefix_command, slash_command, guild_only, category = "Admin")]
async fn max_xp_gain(
    ctx: Context<'_>,
    #[description = "Maximum xp points that can be gained. Leave empty to get the actual value."]
    #[min = 0]
    value: Option<u32>,
) -> Result<(), Error> {
    let Some(guild_id) = ctx.guild_id() else {
        ctx.say("Must be in guild").await?;
        return Ok(());
    };

    if let Some(value) = value {
        ctx.data()
            .db
            .set_max_xp_gain(guild_id.0, value as i64)
            .await?;
    }
    let value = ctx.data().db.get_max_xp_gain(guild_id.0).await?;
    ctx.say(format!("Max Xp gain is set to {value} points."))
        .await?;

    Ok(())
}

/// Timeout a member
///
/// Usage: /tempscalme <@User> <duration (default 60)>
/// duration = 0 to disable timeout
#[instrument(skip(ctx))]
#[poise::command(
    slash_command,
    //required_permissions = "MODERATE_MEMBERS",
    required_bot_permissions = "MODERATE_MEMBERS",
    guild_only,
    category = "Admin"
)]
pub async fn tempscalme(
    ctx: Context<'_>,
    #[description = "User to put in timeout"] mut member: serenity::Member,
    #[description = "Timeout duration (default: 60s)"] duration: Option<i64>,
) -> Result<(), Error> {
    // Cancel timeout
    if let Some(0) = duration {
        member.enable_communication(ctx).await?;

        ctx.say(format!("{} timeout cancelled!", member.mention()))
            .await?;
        info!("timeout cancel");

        return Ok(());
    }

    let now = serenity::Timestamp::now().unix_timestamp();
    let timeout_timestamp = now + duration.unwrap_or(60);
    let time = serenity::Timestamp::from_unix_timestamp(timeout_timestamp)?;

    match member.communication_disabled_until {
        // If to_timestamp > 0, member is already timed out
        Some(to_timestamp) if to_timestamp.unix_timestamp() > now => {
            debug!("to: {} - now: {}", to_timestamp.unix_timestamp(), now);
            info!("already timed out until {}", to_timestamp.naive_local());
            ctx.say(format!(
                "{} is already timed out until {}",
                member.mention(),
                to_timestamp.naive_local()
            ))
            .await?;
        }
        _ => {
            member
                .disable_communication_until_datetime(ctx, time)
                .await?;

            ctx.say(format!(
                "{} timed out until {}",
                member.mention(),
                time.naive_local(),
            ))
            .await?;
            info!(
                "{} timed out until {}",
                member.display_name(),
                time.naive_local()
            );
        }
    }

    Ok(())
}

/// Import users levels from Mee6 leaderboard
#[instrument(skip(ctx))]
#[poise::command(
    slash_command,
    required_permissions = "ADMINISTRATOR",
    guild_only,
    ephemeral,
    category = "Admin"
)]
pub async fn import_mee6_levels(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("This will overwrite current levels. Type \"yes\" to confirm.")
        .await?;

    // Wait for a confirmation from the user
    if let Some(response) = ctx
        .author()
        .await_reply(ctx)
        .timeout(std::time::Duration::from_secs(30))
        .await
    {
        if &response.content == "yes" {
            ctx.say("Ok lesgo!").await?;
        } else {
            ctx.say("ABORT ABORT").await?;
            return Ok(());
        }
    } else {
        ctx.say("I'm not waiting any longer.").await?;
        return Ok(());
    }

    let guild_id = ctx.guild_id().unwrap().0;
    let url = format!("https://mee6.xyz/api/plugins/levels/leaderboard/{guild_id}");

    let text = reqwest::get(url).await?.text().await?;
    let json: serde_json::Value = serde_json::from_str(&text)?;
    let players = json["players"].clone();
    let players = players.as_array().unwrap();

    let user_levels: Vec<_> = players
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let user_id = p["id"].to_string().replace('"', "").parse::<u64>().unwrap();
            let xp = p["detailed_xp"][2].as_i64().unwrap();
            let level = p["level"].as_i64().unwrap();
            let rank = i as i64 + 1;
            let last_message = 0_i64;

            UserLevel {
                user_id,
                xp,
                level,
                rank,
                last_message,
            }
        })
        .collect();

    ctx.data()
        .db
        .import_from_mee6(user_levels, guild_id)
        .await?;

    Ok(())
}
