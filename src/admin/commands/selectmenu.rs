use std::time::Duration;
use poise::{serenity_prelude as serenity, CreateReply};
use tracing::{debug, error, instrument};
use crate::{Context, Error};

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