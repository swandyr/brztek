use std::time::Duration;

use poise::{
    serenity_prelude::{self as serenity, UserId},
    CreateReply,
};
use tracing::{debug, error, info, instrument};

use super::levels::{self, user_level::UserLevel};
use crate::{Context, Data, Error};

/// Admin commands
///
/// Prefix subcommands that need Administrator privileges.
///
/// Available subcommands are set_pub, set_user, spam_delay, min_xp_gain, max_xp_gain.
#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    subcommands("set_user"),
    required_permissions = "ADMINISTRATOR",
    category = "Admin"
)]
pub async fn admin(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Set the user's xp points
#[instrument(skip(ctx))]
#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    ephemeral,
    category = "Admin"
)]
async fn set_user(
    ctx: Context<'_>,
    #[description = "User to modify"] user: serenity::User,
    #[description = "Amount of Xp"]
    #[min = 0]
    xp: u32,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Not in guild")?;
    let user_id = user.id;

    let level = levels::xp_func::calculate_level_from_xp(xp as i64);

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

#[instrument(skip(ctx))]
#[poise::command(slash_command, guild_only, category = "Admin")]
pub async fn selectmenu(ctx: Context<'_>) -> Result<(), Error> {
    // Get list of guild's roles
    let guild_roles = ctx
        .guild()
        .unwrap()
        .roles
        .keys()
        .copied()
        .collect::<Vec<serenity::RoleId>>();
    let max_values = guild_roles.len() as u8;
    // Get list of member's roles
    let member_roles = ctx.author_member().await.unwrap().roles.clone();

    // Create SelectMenu with guild's roles; member's roles are selected by default
    let create_select_menu = serenity::CreateSelectMenu::new(
        "roles_menu",
        serenity::CreateSelectMenuKind::Role {
            default_roles: Some(member_roles),
        },
    )
    .min_values(0)
    .max_values(max_values)
    .placeholder("Role");

    // Send message with SelectMenu; get a message handler to handle interaction
    let m = ctx
        .send(
            CreateReply::default()
                .content("A select menu with roles")
                .components(vec![serenity::CreateActionRow::SelectMenu(
                    create_select_menu,
                )]),
        )
        .await?
        .into_message()
        .await?;

    // Get interaction content (selected roles)
    let interaction = match m
        .await_component_interaction(&ctx.serenity_context().shard)
        .timeout(Duration::from_secs(60 * 3))
        .await
    {
        Some(x) => x,
        None => {
            m.reply(&ctx, "Timed out").await?;
            m.delete(&ctx).await?;
            return Ok(());
        }
    };

    let serenity::ComponentInteractionDataKind::RoleSelect { values } = &interaction.data.kind
    else {
        error!("Invalid ComponentInteractionDataKind");
        m.delete(&ctx).await?;
        return Ok(());
    };

    let roles = values
        .iter()
        .map(|id| id.to_role_cached(ctx).unwrap().name)
        .collect::<Vec<String>>();
    let content = format!("You selected {}", roles.join(", "));
    debug!("{content}");

    // Respond to interaction
    interaction
        .create_response(
            &ctx,
            serenity::CreateInteractionResponse::Message(
                serenity::CreateInteractionResponseMessage::new().content(content),
            ),
        )
        .await?;

    m.delete(&ctx).await?;

    Ok(())
}
