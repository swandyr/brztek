use crate::{Context, Data, Error};

/// Registers slash commands in this guild or globally
#[poise::command(prefix_command, slash_command, ephemeral, hide_in_help, owners_only)]
pub async fn register(ctx: Context<'_>) -> Result<(), Error> {
    poise::builtins::register_application_commands_buttons(ctx).await?;
    Ok(())
}

#[poise::command(prefix_command, slash_command, ephemeral)]
pub async fn help(
    ctx: Context<'_>,
    #[description = "Specific command to show help about"]
    #[autocomplete = "poise::builtins::autocomplete_command"]
    command: Option<String>,
) -> Result<(), Error> {
    poise::builtins::help(
        ctx,
        command.as_deref(),
        poise::builtins::HelpConfiguration {
            extra_text_at_bottom: "\
Type $help command for more info on a command.",
            show_context_menu_commands: true,
            show_subcommands: true,
            include_description: true,
            ephemeral: true,
            ..Default::default()
        },
    )
    .await?;
    Ok(())
}

pub fn all() -> Vec<poise::Command<Data, Error>> {
    vec![register(), help()]
}
