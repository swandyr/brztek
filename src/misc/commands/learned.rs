use tracing::instrument;
use crate::{Context, Error, misc::queries};
use poise::serenity_prelude as serenity;

/// What the bot learned.
///
/// List all learned command names.
#[instrument(skip(ctx))]
#[poise::command(prefix_command, slash_command, guild_only, category = "Misc")]
pub async fn learned(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Not in guild")?.get();
    let db = &ctx.data().db;

    let commands = queries::get_learned_list(db, guild_id).await?;

    let mut content = String::from(">>> List of learned commands: \n");
    let mut content_len = content.len();
    for command in commands {
        let line = format!("- {command}\n");
        content_len += line.len();

        if content_len <= serenity::constants::MESSAGE_CODE_LIMIT {
            // Limit of character accepted in a discord message
            content.push_str(&line);
        } else {
            ctx.say(content).await?;
            content = format!(">>> {line}");
            content_len = content.len();
        }
    }

    ctx.say(content).await?;

    Ok(())
}