use std::collections::HashMap;

use poise::serenity_prelude::{self as serenity, Member};
use poise::serenity_prelude::{CacheHttp, Mentionable};
use rand::{prelude::thread_rng, Rng};
use tracing::{debug, info, instrument};

use crate::{draw::roulette_killfeed::gen_killfeed, Data};

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

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
    category = "Timeouts"
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
            timeout_member(ctx, &mut member, time).await?;

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

/// A random member is in timeout for 60s
#[instrument(skip(ctx))]
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    required_bot_permissions = "MODERATE_MEMBERS",
    category = "Timeouts"
)]
pub async fn roulette(ctx: Context<'_>) -> Result<(), Error> {
    // Retrieves the list of members of the guild
    // let channel = ctx.channel_id().to_channel(ctx).await?.guild().unwrap();
    let guild = ctx.guild().unwrap();
    let members = guild
        .members(ctx, None, None)
        .await?
        .into_iter()
        .filter(|m| !m.user.bot)
        .collect::<Vec<_>>();

    let index = thread_rng().gen_range(0..members.len());
    let mut member = members.get(index).unwrap().clone();
    debug!("Randomly selected member: {:?}", member);

    let now = serenity::Timestamp::now().unix_timestamp();
    let timeout_timestamp = now + 60;
    let time = serenity::Timestamp::from_unix_timestamp(timeout_timestamp)?;

    let timeout_result = timeout_member(ctx, &mut member, time).await;

    let user_1 = ctx.author_member().await.unwrap();

    // Store result in db
    let db = &ctx.data().db;
    let guild_id = guild.id.0;
    let user_1_id = user_1.user.id.0;
    let user_2_id = member.user.id.0;

    db.add_roulette_result(guild_id, now, user_1_id, user_2_id)
        .await?;

    // Reply on the channel
    let user_1_name = user_1
        .display_name()
        .replace(|c: char| !(c.is_alphanumeric() || c.is_whitespace()), "");
    let user_2_name = member
        .display_name()
        .replace(|c: char| !(c.is_alphanumeric() || c.is_whitespace()), "");
    let image = gen_killfeed(&user_1_name, &user_2_name)?;

    ctx.send(|m| {
        let file = serenity::AttachmentType::from((image.as_slice(), "kf.png"));
        m.attachment(file)
    })
    .await?;

    if timeout_result.is_err() {
        ctx.say(format!("The roulette has chosen, {}, but I can't mute you, would you kindly shut up for the next 60 seconds ?", member.mention()))
            .await?;
    }

    if user_1_name == user_2_name {
        ctx.say("https://tenor.com/view/damn-punch-punching-oops-missed-punch-gif-12199143")
            .await?;
    }

    Ok(())
}

#[instrument(skip(ctx))]
async fn timeout_member(
    ctx: Context<'_>,
    member: &mut serenity::Member,
    time: serenity::Timestamp,
) -> Result<(), Error> {
    member
        .disable_communication_until_datetime(ctx, time)
        .await?;

    Ok(())
}

#[instrument(skip(ctx))]
#[poise::command(slash_command, prefix_command, guild_only, category = "Timeouts")]
pub async fn toproulette(ctx: Context<'_>, member: Option<Member>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap().0;

    if let Some(member) = member {
        let user_id = member.user.id.0;
        let targets_fields = member_top_victims(&ctx, guild_id, user_id).await?;
        let title = format!("Top {}'s victims", member.display_name());

        // Send embedded top 10 targets of the user
        ctx.send(|b| b.embed(|f| f.title(title).field("Victims", &targets_fields, true)))
            .await?;
    } else {
        let (callers_field, targets_field) = roulette_leaderboard(&ctx, guild_id).await?;

        // Send embedded top 10 leaderboard
        ctx.send(|b| {
            b.embed(|f| {
                f.title("Roulette Leaderboard")
                    .field("Callers", &callers_field, true)
                    .field("Targets", &targets_field, true)
            })
        })
        .await?;
    }

    Ok(())
}

#[instrument(skip(ctx))]
async fn roulette_leaderboard(
    ctx: &Context<'_>,
    guild_id: u64,
) -> anyhow::Result<(String, String)> {
    let db = &ctx.data().db;
    let scores = db.get_roulette_scores(guild_id).await?;
    info!("Got roulette scores");

    let mut callers_map = HashMap::new();
    let mut targets_map = HashMap::new();

    scores.iter().for_each(|(caller, target)| {
        callers_map
            .entry(caller)
            .and_modify(|x| *x += 1)
            .or_insert(1);
        targets_map
            .entry(target)
            .and_modify(|x| *x += 1)
            .or_insert(1);
    });

    // Process maps
    let callers_field = process_users_map(ctx, guild_id, callers_map).await?;
    let targets_field = process_users_map(ctx, guild_id, targets_map).await?;

    Ok((callers_field, targets_field))
}

#[instrument(skip(ctx))]
async fn member_top_victims(
    ctx: &Context<'_>,
    guild_id: u64,
    user_id: u64,
) -> anyhow::Result<String> {
    let db = &ctx.data().db;
    let scores = db.get_user_roulette_scores(guild_id, user_id).await?;

    let mut targets_map = HashMap::new();
    scores.iter().for_each(|target| {
        targets_map
            .entry(target)
            .and_modify(|x| *x += 1)
            .or_insert(1);
    });

    let targets_field = process_users_map(ctx, guild_id, targets_map).await?;

    Ok(targets_field)
}

#[instrument(skip(ctx))]
async fn process_users_map(
    ctx: &Context<'_>,
    guild_id: u64,
    map: HashMap<&u64, i32>,
) -> anyhow::Result<String> {
    let mut sorted = map
        .iter()
        .map(|(k, v)| (**k, *v))
        .collect::<Vec<(u64, i32)>>();
    sorted.sort_by(|a, b| b.1.cmp(&a.1));

    let now = std::time::Instant::now();
    let nb_users = 10usize;
    let mut field = String::new();
    for user in sorted.iter().take(nb_users) {
        let member = ctx.http().get_member(guild_id, user.0).await?;
        let line = format!("{} - {}\n", member.display_name(), user.1);
        field.push_str(&line);
    }
    let elapsed = now.elapsed().as_millis();
    info!("Processed {nb_users} users in {elapsed} ms");

    Ok(field)
}
