use poise::{
    serenity_prelude::{self as serenity, Role, RoleId},
    CreateReply,
};
use std::time::Duration;
use tracing::{debug, error, instrument};

use super::queries;
use crate::{Context, Error};

#[instrument(skip(ctx))]
#[poise::command(slash_command, guild_only, category = "Mention Roles")]
pub async fn gimmeroles(ctx: Context<'_>) -> Result<(), Error> {
    //TODO: Assign roles to user
    let db = &ctx.data().db;
    let guild_id = ctx.guild_id().ok_or("Not in guild")?;
    let mention_roleids: Vec<RoleId> = queries::get_role_ids(db, guild_id.get())
        .await?
        .into_iter()
        .map(serenity::RoleId::from)
        .collect();
    let max_values = mention_roleids.len() as u8;
    // Get list of member's roles
    let member_roleids = &ctx.author_member().await.unwrap().roles;
    let options: Vec<serenity::CreateSelectMenuOption> = mention_roleids
        .iter()
        .map(|r| {
            let label = r.to_role_cached(ctx).unwrap().name;
            let value = r.get().to_string();
            let default = member_roleids.contains(r);
            serenity::CreateSelectMenuOption::new(label, value).default_selection(default)
        })
        .collect();

    // Create SelectMenu with guild's roles; member's roles are selected by default
    /*let select_menu = serenity::CreateSelectMenu::new(
        "roles_menu",
        serenity::CreateSelectMenuKind::Role {
            default_roles: Some(member_roles),
        },
    )
    .min_values(0)
    .max_values(max_values)
    .placeholder("Role");*/
    let select_menu = serenity::CreateSelectMenu::new(
        "roles_menu",
        serenity::CreateSelectMenuKind::String { options },
    )
    .min_values(0)
    .max_values(max_values)
    .placeholder("Role");

    // Send message with SelectMenu; get a message handler to handle interaction
    let m = ctx
        .send(
            CreateReply::default()
                .content("A select menu with roles")
                .components(vec![serenity::CreateActionRow::SelectMenu(select_menu)]),
        )
        .await?
        .into_message()
        .await?;

    // Get interaction content (selected roles)
    let Some(interaction) = m
        .await_component_interaction(&ctx.serenity_context().shard)
        .timeout(Duration::from_secs(60 * 3))
        .await
    else {
        m.reply(&ctx, "Timed out").await?;
        m.delete(&ctx).await?;
        return Ok(());
    };

    let serenity::ComponentInteractionDataKind::StringSelect { values } = &interaction.data.kind
    else {
        error!("Invalid ComponentInteractionDataKind");
        m.delete(&ctx).await?;
        return Ok(());
    };

    let roles = values
        .iter()
        .map(|value| {
            let id = RoleId::from(value.parse::<u64>().unwrap());
            id.to_role_cached(ctx).unwrap().name
        })
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
