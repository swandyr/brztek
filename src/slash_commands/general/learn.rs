pub use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::{
        command::CommandOptionType,
        interaction::application_command::{CommandDataOption, CommandDataOptionValue},
    },
    prelude::Context,
};

pub use crate::utils::db::Db;

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("learn")
        .description("Save a link with a name")
        .create_option(|option| {
            option
                .name("name")
                .description("A name")
                .kind(CommandOptionType::String)
                .required(true)
        })
        .create_option(|option| {
            option
                .name("link")
                .description("A link")
                .kind(CommandOptionType::String)
                .required(true)
        })
}

pub async fn run(ctx: &Context, options: &[CommandDataOption]) -> String {
    let options = (
        options.get(0).expect("no name").resolved.as_ref().unwrap(),
        options.get(1).expect("no link").resolved.as_ref().unwrap(),
    );

    if let (CommandDataOptionValue::String(name), CommandDataOptionValue::String(link)) = options {
        let data = ctx.data.read().await;
        let db = data.get::<Db>().unwrap();
        db.learn_command(name, link).await.unwrap();

        "Command learned!".to_string()
    } else {
        "Something gone wrong :(".to_string()
    }
}
