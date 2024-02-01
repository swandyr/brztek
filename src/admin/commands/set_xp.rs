use tracing::{instrument, info};
use poise::serenity_prelude as serenity;
use crate::{Context, Error, levels};

/// Set the user's xp points
#[instrument(skip(ctx))]
#[poise::command(
prefix_command,
slash_command,
guild_only,
ephemeral,
required_permissions = "ADMINISTRATOR",
category = "Admin"
)]
pub async fn set_xp(
    ctx: Context<'_>,
    #[description = "User to modify"] user: serenity::User,
    #[description = "Amount of Xp"]
    #[min = 0]
    xp: u32,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Not in guild")?;
    let user_id = user.id;

    let level = levels::func::xp_func::calculate_level_from_xp(xp as i64);

    let db = ctx.data().db.as_ref();
    let mut user_level = levels::queries::get_user(db, user_id.get(), guild_id.get()).await?;
    user_level.xp = xp as i64;
    user_level.level = level;
    levels::queries::update_user(db, &user_level, guild_id.get()).await?;

    info!("Admin updated user {user_id} in guild {guild_id}: {xp} - {level}");

    ctx.say(format!("{} is now level {}", user.name, user_level.level))
        .await?;

    Ok(())
}