mod draw;
mod queries;

const BASE_RFF_PERC: u8 = 5;

use std::collections::HashMap;

use poise::{
    serenity_prelude::{self as serenity, CreateMessage, Guild, Member, Mentionable, UserId},
    CreateReply,
};
use rand::{prelude::thread_rng, Rng};
use tracing::{debug, info, instrument, warn};

use super::to_png_buffer;
use crate::{Context, Data, Db, Error};

#[derive(Debug)]
pub enum ShotKind {
    Normal,
    SelfShot,
    Reverse,
}

#[derive(Debug, Clone, Copy)]
pub struct Roulette {
    pub timestamp: i64,
    pub caller_id: UserId,
    pub target_id: UserId,
    // Is Some if the record has triggered rff, is None if it processed normally
    pub rff_triggered: Option<u8>,
}

/// Put random member in timeout for 60s
///
/// The more you use it, the more you can get caught
#[instrument(skip(ctx))]
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    required_bot_permissions = "MODERATE_MEMBERS",
    category = "Roulette"
)]
pub async fn roulette(ctx: Context<'_>) -> Result<(), Error> {
    let mut author = ctx
        .author_member()
        .await
        .ok_or("No author_member found")?
        .into_owned();
    let author_id = author.user.id;

    let now = serenity::Timestamp::now().unix_timestamp();
    let timeout_timestamp = now + 60;
    let time = serenity::Timestamp::from_unix_timestamp(timeout_timestamp)?;

    let db = &ctx.data().db;

    let rff_check = thread_rng().gen_range(1..=100);
    debug!("Generated RFF Check: {}", rff_check);

    let mut entry = {
        let read = ctx.data().roulette_map.lock().unwrap();
        debug!("{read:#?}");
        read.get(&author_id).copied()
    };
    debug!(
        "Roulette Map entry of user {}: {:?}",
        author.display_name(),
        entry
    );
    let (rff_user_chance, _timestamp) = entry.get_or_insert((BASE_RFF_PERC, now));
    debug!("RFF user chance: {}", rff_user_chance);

    //TODO: reset under a certain period of time ?

    // Author is self timed out if the random rff_check number is below the author's rff_user_chance
    if rff_check <= *rff_user_chance {
        info!("Selfshot check not passed for {}", author.display_name());
        // Returns error if the bot cannot timeout_member, usually because he has administrator status
        let timeout_result = timeout_member(ctx, &mut author, time).await;
        // Saves result in db
        let roulette = Roulette {
            timestamp: now,
            caller_id: author_id,
            target_id: author_id,
            rff_triggered: Some(*rff_user_chance),
        };
        debug!("{:#?}", roulette);
        let guild = ctx.guild().as_deref().ok_or("Not in guild")?.clone();
        record_roulette(db, &guild, roulette).await?;
        // Generates the image that will be attached to the message
        let image = gen_roulette_image(&author, &author, ShotKind::Reverse).await?;

        let file = serenity::CreateAttachment::bytes(image.as_slice(), "kf.png");
        let content = format!("**:man_police_officer: RFF activated at {rff_user_chance}%, you're out. :woman_police_officer:**");
        ctx.send(CreateReply::default().attachment(file).content(content))
            .await?;

        // if timeout_member returned Err, it assumes it is because of administrator priviledges, then notify the member
        if let Err(e) = timeout_result {
            warn!("Timeout member returned: {}", e);
            ctx.say(
                "As you're an administrator, I have no power, but I know you won't abuse the rules",
            )
            .await?;
        }

        // Reset the author's selfshot_perc
        {
            let mut write = ctx.data().roulette_map.lock().unwrap();
            write.entry(author_id).and_modify(|(perc, tstamp)| {
                *perc = BASE_RFF_PERC;
                *tstamp = now;
            });
        }
    } else {
        // Get a random member
        let guild = ctx.guild().as_deref().ok_or("Not in guild")?.clone();
        let members = guild
            .members(ctx, None, None)
            .await?
            .into_iter()
            .filter(|m| !m.user.bot)
            .collect::<Vec<_>>();
        let index = thread_rng().gen_range(0..members.len());
        let mut target = members.get(index).ok_or("No member found")?.clone();
        info!("Randomly selected member: {:?}", target.display_name());

        let timeout_result = timeout_member(ctx, &mut target, time).await;
        let roulette = Roulette {
            timestamp: now,
            caller_id: author_id,
            target_id: target.user.id,
            rff_triggered: None,
        };
        record_roulette(db, &guild, roulette).await?;

        let is_self_shot = author_id == target.user.id.get();

        let image = gen_roulette_image(
            &author,
            &target,
            if is_self_shot {
                ShotKind::SelfShot
            } else {
                ShotKind::Normal
            },
        )
        .await?;

        // Send a message according to a self shot or not
        if is_self_shot {
            ctx.send(
                CreateReply::default()
                    .attachment(serenity::CreateAttachment::bytes(
                        image.as_slice(),
                        "kf.png",
                    ))
                    .content(
                        //"https://tenor.com/view/damn-punch-punching-oops-missed-punch-gif-12199143",
                        "Ouch, looks like it hurts. :sweat_smile:",
                    ),
            )
            .await?;
        } else {
            ctx.send(
                CreateReply::default().attachment(serenity::CreateAttachment::bytes(
                    image.as_slice(),
                    "kf.png",
                )),
            )
            .await?;
        }

        if let Err(e) = timeout_result {
            warn!("Timeout member returned: {}", e);
            ctx.say(format!("The roulette has chosen, {}, but I can't mute you, would you kindly shut up for the next 60 seconds ?", target.mention()))
                .await?;
        }

        // Increase author's rff_user_chance
        {
            let inc = thread_rng().gen_range(2..11);
            let mut write = ctx.data().roulette_map.lock().unwrap();
            write
                .entry(author_id)
                .and_modify(|(rff_perc, tstamp)| {
                    *rff_perc.clamp(&mut 0, &mut 100) += inc;
                    *tstamp = now;
                })
                .or_insert((BASE_RFF_PERC + inc, now));
        }
    }

    Ok(())
}

#[instrument(skip_all)]
async fn record_roulette(db: &Db, guild: &Guild, roulette: Roulette) -> Result<(), Error> {
    let guild_id = guild.id.get();

    queries::add_roulette_result(db, guild_id, roulette).await?;

    Ok(())
}

#[instrument(skip_all)]
async fn gen_roulette_image(
    author: &Member,
    target: &Member,
    kind: ShotKind,
) -> Result<Vec<u8>, Error> {
    let author_name = author
        .display_name()
        .replace(|c: char| !(c.is_alphanumeric() || c.is_whitespace()), "");
    let target_name = target
        .display_name()
        .replace(|c: char| !(c.is_alphanumeric() || c.is_whitespace()), "");

    draw::gen_killfeed(&author_name, &target_name, kind)
}

#[instrument(skip(ctx))]
async fn timeout_member(
    ctx: Context<'_>,
    member: &mut Member,
    time: serenity::Timestamp,
) -> Result<(), Error> {
    member
        .disable_communication_until_datetime(ctx, time)
        .await?;

    Ok(())
}

/// Roulette Leaderboard
///
/// Shows the top 10 users and top 10 targets of the server
#[instrument(skip(ctx))]
#[poise::command(slash_command, prefix_command, guild_only, category = "Roulette")]
pub async fn toproulette(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Not in guild")?.get();

    let db = &ctx.data().db;
    let scores = queries::get_roulette_scores(db, guild_id).await?;

    let mut callers_map = HashMap::new();
    let mut targets_map = HashMap::new();
    let mut rff_map = HashMap::new();

    for score in &scores {
        callers_map
            .entry(score.caller_id)
            .and_modify(|x| *x += 1)
            .or_insert(1);
        targets_map
            .entry(score.target_id)
            .and_modify(|x| *x += 1)
            .or_insert(1);
        if let Some(rff) = score.rff_triggered {
            rff_map
                .entry(score.caller_id)
                .and_modify(|x| {
                    if *x < rff.into() {
                        *x = rff.into();
                    }
                })
                .or_insert(rff.into());
        }
    }

    // Process maps
    let callers_field = process_users_map(&ctx, callers_map).await?;
    let targets_field = process_users_map(&ctx, targets_map).await?;
    let rff_fields = process_users_map(&ctx, rff_map).await?;

    // Send embedded top 10 leaderboard
    ctx.send(
        CreateReply::default().embed(
            serenity::CreateEmbed::new()
                .title("Roulette Leaderboard")
                .field("Callers", &callers_field, true)
                .field("Targets", &targets_field, true)
                .field("max RFF%", &rff_fields, true),
        ),
    )
    .await?;

    Ok(())
}

/// Shows some statistics about the use of roulettes
#[instrument(skip(ctx, member))]
#[poise::command(slash_command, prefix_command, guild_only, category = "Roulette")]
pub async fn statroulette(ctx: Context<'_>, member: Option<Member>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Not in guild")?.get();
    let member = member.unwrap_or(
        ctx.author_member()
            .await
            .ok_or("No member found")?
            .into_owned(),
    );
    let member_id = member.user.id;

    let db = &ctx.data().db;
    let scores = queries::get_roulette_scores(db, guild_id).await?;

    // Stats
    let member_scores = scores
        .iter()
        .filter(|score| score.caller_id == member_id.get())
        .collect::<Vec<&Roulette>>();
    let total_member_shots = member_scores.len();
    let total_member_selfshots = member_scores
        .iter()
        .filter(|score| score.target_id == member_id.get() && score.rff_triggered.is_none())
        .count();
    let total_member_rff_triggered = member_scores
        .iter()
        .filter(|score| score.target_id == member_id.get() && score.rff_triggered.is_some())
        .count();
    let member_rff_perc = {
        let map = ctx.data().roulette_map.lock().unwrap();
        map.get(&member_id).unwrap_or(&(BASE_RFF_PERC, 0)).0
    };
    let max_member_rff_perc = member_scores
        .iter()
        .filter(|score| score.target_id == member_id.get())
        .filter_map(|score| score.rff_triggered)
        .max();
    let min_member_rff_triggered = member_scores
        .iter()
        .filter(|score| score.target_id == member_id.get())
        .filter_map(|score| score.rff_triggered)
        .min();

    let stats_field = format!(
        r#"{} roulettes
{} selfshots
{} RFF triggered
{}% chance of RFF
{}% max RFF triggered
{}% min RFF triggered"#,
        total_member_shots,
        total_member_selfshots,
        total_member_rff_triggered,
        member_rff_perc,
        max_member_rff_perc.unwrap_or(0),
        min_member_rff_triggered.unwrap_or(0),
    );

    // Top victims
    let mut targets_map = HashMap::new();
    scores
        .iter()
        .filter(|record| record.caller_id == member_id && record.rff_triggered.is_none())
        .for_each(|record| {
            targets_map
                .entry(record.target_id)
                .and_modify(|x| *x += 1)
                .or_insert(1);
        });

    let targets_field = process_users_map(&ctx, targets_map).await?;

    // Top bullies
    let mut bullies_map = HashMap::new();
    scores
        .iter()
        .filter(|record| record.target_id == member_id && record.rff_triggered.is_none())
        .for_each(|record| {
            bullies_map
                .entry(record.caller_id)
                .and_modify(|x| *x += 1)
                .or_insert(1);
        });
    let bullies_field = process_users_map(&ctx, bullies_map).await?;

    ctx.send(
        CreateReply::default().embed(
            serenity::CreateEmbed::new()
                .title(member.display_name())
                .field("Stats", stats_field, true)
                .field("Victims", targets_field, true)
                .field("Bullies", bullies_field, true),
        ),
    )
    .await?;

    Ok(())
}

/// Who goes the highest before trigerring RFF ?
#[instrument(skip(ctx))]
#[poise::command(slash_command, prefix_command, guild_only, category = "Roulette")]
pub async fn rffstar(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Not in guild")?.get();

    let db = &ctx.data().db;
    let scores = queries::get_roulette_scores(db, guild_id).await?;
    let rff_score = scores.iter().max_by_key(|record| record.rff_triggered);
    let guild = ctx.guild().ok_or("Not in guild")?.clone();

    if let Some(record) = rff_score {
        if let Some(score) = record.rff_triggered {
            let mention = guild.member(ctx, record.caller_id).await?.mention();
            ctx.say(format!(
                ":muscle: :military_medal: {mention} is the RFF Star with {score}%."
            ))
            .await?;

            return Ok(());
        }
    }

    ctx.say("Nobody has triggered the RFF yet.").await?;

    Ok(())
}

#[instrument(skip(ctx, map))]
async fn process_users_map(ctx: &Context<'_>, map: HashMap<UserId, i32>) -> Result<String, Error> {
    let mut sorted = map
        .iter()
        .map(|(k, v)| (*k, *v))
        .collect::<Vec<(UserId, i32)>>();
    sorted.sort_by(|a, b| b.1.cmp(&a.1));

    let now = std::time::Instant::now();

    let guild_members = &ctx.guild().ok_or("Not in guild")?.members;
    let nb_users = 5usize;
    let mut field = String::new();
    for user in sorted.iter().take(nb_users) {
        //let member = ctx.http().get_member(guild_id, user.0).await?;
        let member = guild_members.get(&user.0).ok_or("No member found")?;
        let line = format!("{} - {}\n", member.display_name(), user.1);
        field.push_str(&line);
    }
    let elapsed = now.elapsed().as_millis();
    info!("Processed {nb_users} users in {elapsed} ms");

    Ok(field)
}
