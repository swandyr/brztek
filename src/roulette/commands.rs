use std::collections::HashMap;

use poise::serenity_prelude::{self as serenity, Guild, Member, Mentionable, UserId};
use rand::{prelude::thread_rng, Rng};
use tracing::{debug, info, instrument};

use super::{draw, queries, BASE_RFF_PERC};
use crate::{Data, Db};

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

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
    let mut author = ctx.author_member().await.unwrap().into_owned();
    let author_id = author.user.id;

    let now = serenity::Timestamp::now().unix_timestamp();
    let timeout_timestamp = now + 60;
    let time = serenity::Timestamp::from_unix_timestamp(timeout_timestamp)?;

    let db = &ctx.data().db;

    let rff_check = thread_rng().gen_range(1..=100);

    let mut entry = {
        let read = ctx.data().roulette_map.read().unwrap();
        debug!("{read:#?}");
        read.get(&author_id).copied()
    };
    debug!(
        "Cooldown Map entry of user {}: {:?}",
        author.display_name(),
        entry
    );
    let (rff_user_chance, _timestamp) = entry.get_or_insert((BASE_RFF_PERC, now));

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
        record_roulette(db, &ctx.guild().unwrap(), roulette).await?;
        // Generates the image that will be attached to the message
        let image = gen_roulette_image(&author, &author, ShotKind::Reverse).await?;

        ctx.send(|m| {
            let file = serenity::AttachmentType::from((image.as_slice(), "kf.png"));
            let content = format!("**:man_police_officer: RFF activated at {rff_user_chance}%, you're out. :woman_police_officer:**");
            m.attachment(file).content(content)
        })
        .await?;

        // if timeout_member returned Err, it assumes it is because of administrator priviledges, then notify the member
        if timeout_result.is_err() {
            ctx.say(
                "As you're an administrator, I have no power, but I know you won't abuse the rules",
            )
            .await?;
        }

        // Reset the author's selfshot_perc
        {
            let mut write = ctx.data().roulette_map.write().unwrap();
            write.entry(author_id).and_modify(|(perc, tstamp)| {
                *perc = BASE_RFF_PERC;
                *tstamp = now;
            });
        }
    } else {
        // Get a random member
        let guild = ctx.guild().unwrap();
        let members = guild
            .members(ctx, None, None)
            .await?
            .into_iter()
            .filter(|m| !m.user.bot)
            .collect::<Vec<_>>();
        let index = thread_rng().gen_range(0..members.len());
        let mut target = members.get(index).unwrap().clone();
        info!("Randomly selected member: {:?}", target.display_name());

        let timeout_result = timeout_member(ctx, &mut target, time).await;
        let roulette = Roulette {
            timestamp: now,
            caller_id: author_id,
            target_id: target.user.id,
            rff_triggered: None,
        };
        record_roulette(db, &guild, roulette).await?;

        let is_self_shot = author_id == target.user.id.0;

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
            ctx.send(|m| {
                m.attachment(serenity::AttachmentType::from((image.as_slice(), "kf.png")))
                    .content(
                        //"https://tenor.com/view/damn-punch-punching-oops-missed-punch-gif-12199143",
                        "Ouch, looks like it hurts. :sweat_smile:",
                    )
            })
            .await?;
        } else {
            ctx.send(|m| {
                m.attachment(serenity::AttachmentType::from((image.as_slice(), "kf.png")))
            })
            .await?;
        }

        if timeout_result.is_err() {
            ctx.say(format!("The roulette has chosen, {}, but I can't mute you, would you kindly shut up for the next 60 seconds ?", target.mention()))
            .await?;
        }

        // Increase author's rff_user_chance
        {
            let inc = thread_rng().gen_range(2..11);
            let mut write = ctx.data().roulette_map.write().unwrap();
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
    let guild_id = guild.id.0;

    queries::add_roulette_result(db, guild_id, roulette).await?;

    Ok(())
}

#[instrument(skip_all)]
async fn gen_roulette_image(
    author: &Member,
    target: &Member,
    kind: ShotKind,
) -> Result<Vec<u8>, anyhow::Error> {
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
    let guild_id = ctx.guild_id().unwrap().0;

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
                        *x = rff.into()
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
    ctx.send(|b| {
        b.embed(|f| {
            f.title("Roulette Leaderboard")
                .field("Callers", &callers_field, true)
                .field("Targets", &targets_field, true)
                .field("max RFF%", &rff_fields, true)
        })
    })
    .await?;

    Ok(())
}

/// Shows some statistics about the use of roulettes
#[instrument(skip(ctx, member))]
#[poise::command(slash_command, prefix_command, guild_only, category = "Roulette")]
pub async fn statroulette(ctx: Context<'_>, member: Option<Member>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap().0;
    let member = member.unwrap_or(ctx.author_member().await.unwrap().into_owned());
    let member_id = member.user.id;

    let db = &ctx.data().db;
    let scores = queries::get_roulette_scores(db, guild_id).await?;

    // Stats
    let member_scores = scores
        .iter()
        .filter(|score| score.caller_id == member_id.0)
        .collect::<Vec<&Roulette>>();
    let total_member_shots = member_scores.len();
    let total_member_selfshots = member_scores
        .iter()
        .filter(|score| score.target_id == member_id.0 && score.rff_triggered.is_none())
        .count();
    let total_member_rff_triggered = member_scores
        .iter()
        .filter(|score| score.target_id == member_id.0 && score.rff_triggered.is_some())
        .count();
    let member_rff_perc = {
        let map = ctx.data().roulette_map.read().unwrap();
        map.get(&member_id).unwrap_or(&(BASE_RFF_PERC, 0)).0
    };
    let max_member_rff_perc = member_scores
        .iter()
        .filter(|score| score.target_id == member_id.0)
        .filter_map(|score| score.rff_triggered)
        .max();
    let min_member_rff_triggered = member_scores
        .iter()
        .filter(|score| score.target_id == member_id.0)
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

    ctx.send(|b| {
        b.embed(|f| {
            f.title(member.display_name())
                .field("Stats", stats_field, true)
                .field("Victims", targets_field, true)
                .field("Bullies", bullies_field, true)
        })
    })
    .await?;

    Ok(())
}

/// Who goes the highest before trigerring RFF ?
#[instrument(skip(ctx))]
#[poise::command(slash_command, prefix_command, guild_only, category = "Roulette")]
pub async fn rffstar(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap().0;

    let db = &ctx.data().db;
    let scores = queries::get_roulette_scores(db, guild_id).await?;
    let rff_score = scores.iter().max_by_key(|record| record.rff_triggered);

    if let Some(record) = rff_score {
        if let Some(score) = record.rff_triggered {
            let mention = ctx
                .guild()
                .unwrap()
                .member(ctx, record.caller_id)
                .await?
                .mention();
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
async fn process_users_map(ctx: &Context<'_>, map: HashMap<UserId, i32>) -> anyhow::Result<String> {
    let mut sorted = map
        .iter()
        .map(|(k, v)| (*k, *v))
        .collect::<Vec<(UserId, i32)>>();
    sorted.sort_by(|a, b| b.1.cmp(&a.1));

    let now = std::time::Instant::now();

    let guild_members = ctx.guild().unwrap().members;
    let nb_users = 5usize;
    let mut field = String::new();
    for user in sorted.iter().take(nb_users) {
        //let member = ctx.http().get_member(guild_id, user.0).await?;
        let member = guild_members.get(&user.0.into()).unwrap();
        let line = format!("{} - {}\n", member.display_name(), user.1);
        field.push_str(&line);
    }
    let elapsed = now.elapsed().as_millis();
    info!("Processed {nb_users} users in {elapsed} ms");

    Ok(field)
}
