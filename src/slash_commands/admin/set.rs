use crate::utils::{
    db::Db,
    levels::{user_level::UserLevel, xp::calculate_level_from_xp},
};
use serenity::{
    builder::CreateApplicationCommand,
    model::{
        guild,
        prelude::{
            command::CommandOptionType,
            interaction::application_command::{CommandDataOption, CommandDataOptionValue},
            GuildId,
        },
        Permissions,
    },
    prelude::Context,
};

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("set")
        .description("Set xp points & messages count to a user.")
        .default_member_permissions(Permissions::ADMINISTRATOR)
        .create_option(|option| {
            option
                .name("messages")
                .description("Number of messages")
                .kind(CommandOptionType::Integer)
                .required(true)
        })
        .create_option(|option| {
            option
                .name("xp")
                .description("Experience points")
                .kind(CommandOptionType::Integer)
                .required(true)
        })
        .create_option(|option| {
            option
                .name("user")
                .description("Who ?")
                .kind(CommandOptionType::User)
                .required(true)
        })
}

pub async fn run(ctx: &Context, options: &[CommandDataOption], guild_id: &GuildId) -> String {
    let data = ctx.data.read().await;
    let db = data.get::<Db>().unwrap();
    let guild_id = guild_id.0;

    let option_messages = options
        .get(0)
        .expect("no messages")
        .resolved
        .as_ref()
        .unwrap();
    let messages = if let CommandDataOptionValue::Integer(value) = option_messages {
        *value
    } else {
        return "Please provide a valid value".to_string();
    };

    let option_xp = options.get(1).expect("no xp").resolved.as_ref().unwrap();
    let xp = if let CommandDataOptionValue::Integer(value) = option_xp {
        *value
    } else {
        return "Please provide a valid parameter.".to_string();
    };

    let option_user = options.get(2).expect("no user").resolved.as_ref().unwrap();
    let user_id = if let CommandDataOptionValue::User(value, _) = option_user {
        value.id.0
    } else {
        return "Please provide a valid user".to_string();
    };

    let level = calculate_level_from_xp(xp);

    let mut user = db.get_user(user_id, guild_id).await.unwrap();
    user.xp = xp;
    user.level = level;
    user.messages = messages;
    db.update_user(&user, guild_id).await.unwrap();

    "Updated.".to_string()
}
