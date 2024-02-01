use poise::{serenity_prelude as serenity, CreateReply};
use std::collections::HashMap;
use tracing::instrument;

use super::{consts::BASE_RFF_PERC, func, models::Roulette, queries};
use crate::{Context, Error};

/// Shows some statistics about the use of roulettes
#[instrument(skip(ctx, member))]
#[poise::command(slash_command, prefix_command, guild_only, category = "Roulette")]
pub async fn statroulette(ctx: Context<'_>, member: Option<serenity::Member>) -> Result<(), Error> {
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

    let targets_field = func::process_users_map(&ctx, targets_map).await?;

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
    let bullies_field = func::process_users_map(&ctx, bullies_map).await?;

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
