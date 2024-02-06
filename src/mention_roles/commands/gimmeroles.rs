use poise::{
    serenity_prelude::{self as serenity, Role, RoleId},
    CreateReply,
};
use std::time::Duration;
use tracing::{debug, error, instrument};

use super::{queries, util};
use crate::{Context, Error};

/// Get roles to be mentionned
#[instrument(skip(ctx))]
#[poise::command(slash_command, guild_only, category = "Mention Roles")]
pub async fn gimmeroles(ctx: Context<'_>) -> Result<(), Error> {
    let db = &ctx.data().db;
    let guild_id = ctx.guild_id().ok_or("Not in guild")?;
    let mention_roleids: Vec<RoleId> = queries::get_role_ids(db, guild_id.get())
        .await?
        .into_iter()
        .map(serenity::RoleId::from)
        .collect();
    let max_values = mention_roleids.len() as u8;
    let member_roleids = &ctx.author_member().await.unwrap().roles;

    // Create select menu entries; roles already assigned to user is selected by default
    let options: Vec<serenity::CreateSelectMenuOption> = mention_roleids
        .iter()
        .map(|id| {
            let label = id.to_role_cached(ctx).unwrap().name;
            let value = id.get().to_string();
            let default = member_roleids.contains(id);
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

    // Assign selected roles to user
    let member = ctx.author_member().await.ok_or("author_member not found")?;
    let selected = values
        .iter()
        .map(|value| RoleId::from(value.parse::<u64>().unwrap()))
        .collect::<Vec<RoleId>>();
    member.add_roles(ctx, &selected).await?;
    let unselected = mention_roleids
        .into_iter()
        .filter(|r| !selected.contains(r) && member_roleids.contains(r))
        .collect::<Vec<RoleId>>();
    member.remove_roles(ctx, &unselected).await?;

    // Send response
    let selected_names = selected
        .iter()
        .map(|r| r.to_role_cached(ctx).unwrap().name)
        .collect::<Vec<String>>();
    let unselected_names = unselected
        .iter()
        .map(|r| r.to_role_cached(ctx).unwrap().name)
        .collect::<Vec<String>>();
    let content = format!(
        "Roles assigned: {}\nRoles removed: {}",
        selected_names.join(", "),
        unselected_names.join(", ")
    );
    debug!("{content}");

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
