use serenity::{
    builder::CreateApplicationCommand,
    model::{
        prelude::{
            command::CommandOptionType,
            interaction::application_command::{CommandDataOption, CommandDataOptionValue},
            GuildId,
        },
        Permissions,
    },
    prelude::Context,
};

use crate::utils::db::Db;

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("pub")
        .description("Set the channel where to send Welcome messages")
        .default_member_permissions(Permissions::ADMINISTRATOR)
        .create_option(|option| {
            option
                .name("channel")
                .description("Name of the channel")
                .kind(CommandOptionType::Channel)
                .required(true)
        })
}

pub async fn run(ctx: &Context, options: &[CommandDataOption], guild_id: &GuildId) -> String {
    let data = ctx.data.read().await;
    let db = data.get::<Db>().unwrap();
    let guild_id = guild_id.0;

    let option_channel = options
        .get(0)
        .expect("no channel")
        .resolved
        .as_ref()
        .unwrap();
    let channel = if let CommandDataOptionValue::Channel(value) = option_channel {
        value
    } else {
        return "Invalid value".to_string();
    };

    let channel_id = channel.id.0;
    db.set_pub_channel_id(channel_id, guild_id).await.unwrap();

    format!(
        "#{} is the new public channel",
        channel.name.as_ref().unwrap()
    )
}
