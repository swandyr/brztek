use std::collections::HashMap;
use poise::serenity_prelude::{self as serenity, Guild, Member, UserId};
use tracing::{info, instrument};
use crate::database::Db;
use crate::{Context, Error};
use crate::roulette::{draw, queries, models::{Roulette, ShotKind}};

#[instrument(skip_all)]
pub async fn record_roulette(db: &Db, guild: &Guild, roulette: Roulette) -> Result<(), Error> {
    let guild_id = guild.id.get();

    queries::add_roulette_result(db, guild_id, roulette).await?;

    Ok(())
}

#[instrument(skip_all)]
pub async fn gen_roulette_image(
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
pub async fn timeout_member(
    ctx: Context<'_>,
    member: &mut Member,
    time: serenity::Timestamp,
) -> Result<(), Error> {
    member
        .disable_communication_until_datetime(ctx, time)
        .await?;

    Ok(())
}

#[instrument(skip(ctx, map))]
pub async fn process_users_map(ctx: &Context<'_>, map: HashMap<UserId, i32>) -> Result<String, Error> {
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