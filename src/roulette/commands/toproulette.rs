use poise::{serenity_prelude as serenity, CreateReply};
use std::collections::HashMap;
use tracing::instrument;

use super::{func, queries};
use crate::{Context, Error};

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
                .or_insert_with(|| rff.into());
        }
    }

    // Process maps
    let callers_field = func::process_users_map(&ctx, callers_map).await?;
    let targets_field = func::process_users_map(&ctx, targets_map).await?;
    let rff_fields = func::process_users_map(&ctx, rff_map).await?;

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
