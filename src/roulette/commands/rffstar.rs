use poise::serenity_prelude::Mentionable;
use tracing::instrument;

use super::queries;
use crate::{Context, Error};

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
