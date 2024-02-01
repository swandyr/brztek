use poise::serenity_prelude::UserId;
use tracing::instrument;

use crate::{
    levels::{self, models::UserLevel},
    Context, Error,
};

/// Import users levels from Mee6 leaderboard
#[allow(clippy::cast_possible_wrap)]
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

    let guild_id = ctx.guild_id().ok_or("Not in guild")?.get();
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
            let user_id = UserId::from(user_id);
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

    let db = &ctx.data().db;
    levels::queries::import_from_mee6(db, user_levels, guild_id).await?;

    Ok(())
}
