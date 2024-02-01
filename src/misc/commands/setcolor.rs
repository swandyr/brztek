use poise::{serenity_prelude as serenity, CreateReply};
use tracing::{info, instrument};

use super::queries;
use crate::{Context, Error};

/// Get your own role
///
/// Get a personal role with the color of your choice
///
/// Usage: /setcolor <color>
/// where color is in hexadecimal format (eg: #d917d3)
///
/// If no color is given, it will retrieve the profile's banner color
#[instrument(skip(ctx))]
#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    required_bot_permissions = "MANAGE_ROLES",
    ephemeral,
    category = "Misc"
)]
pub async fn setcolor(
    ctx: Context<'_>,
    #[description = "Colour in hexadecimal format"] hex_colour: Option<String>,
) -> Result<(), Error> {
    // Request db for an `Option<u64>` if a role is already attributed to the user
    let db = &ctx.data().db;
    let guild = ctx.guild().as_deref().cloned().ok_or("Not in guild")?;
    let guild_id = guild.id.get();
    let mut member = ctx.author_member().await.ok_or("author_member not found")?;
    let user_id = member.user.id.get();
    let role_id = queries::get_role_color(db, guild_id, user_id).await?;

    // Member display name will be the name of the role
    let name = member.display_name();
    let role_name = format!("bot_color_{name}");

    let colour = if let Some(hex) = hex_colour {
        if !(hex.len() == 7 && hex.starts_with('#')) {
            ctx.say("Color format should be \"#rrggbb\"".to_string())
                .await?;
            return Ok(());
        }

        if !(hex[1..7].chars().all(|c| c.is_ascii_hexdigit())) {
            ctx.say(format!("{hex} is not a valid color hex code."))
                .await?;
            return Ok(());
        }

        let r: u8 = u8::from_str_radix(&hex[1..3], 16)?;
        let g: u8 = u8::from_str_radix(&hex[3..5], 16)?;
        let b: u8 = u8::from_str_radix(&hex[5..7], 16)?;

        serenity::Colour::from_rgb(r, g, b)
    } else {
        // User banner colour will be the colour of the role
        let Some(colour) = ctx.http().get_user(user_id.into()).await?.accent_colour else {
            ctx.say("Cannot find banner color").await?;
            return Ok(());
        };
        colour
    };

    info!("role_id: {:?}", role_id);
    if let Some(id) = role_id {
        guild
            .edit_role(
                ctx,
                id,
                serenity::EditRole::new()
                    .colour(colour.0 as u64)
                    .name(role_name),
            )
            .await?;
        info!("role_color {} updated", id);
    } else {
        let bot_role_position = guild.role_by_name("brztek").unwrap().position;
        info!("bot role position: {}", bot_role_position);
        let role = guild
            .create_role(
                ctx,
                serenity::EditRole::new()
                    .name(role_name)
                    .colour(colour.0 as u64)
                    .permissions(serenity::Permissions::empty())
                    .position(bot_role_position - 1),
            )
            .await?;
        info!("role_color created: {}", role.id.get());

        // Add the role to the user
        member.to_mut().add_role(ctx, role.id).await?;
        info!("role added to user");

        let role_id = role.id.get();
        queries::set_role_color(db, guild_id, user_id, role_id).await?;
    }

    ctx.send(CreateReply::default().reply(true).content("Done!"))
        .await?;

    Ok(())
}
