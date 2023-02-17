use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
    prelude::*,
};

use crate::utils::db::Db;

#[command]
pub async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id
        .send_message(&ctx.http, |m| m.content("Pong!"))
        .await?;
    Ok(())
}

#[command]
pub async fn learn(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let command_name = args.single::<String>();
    let command_link = args.single::<String>();

    if command_link.is_err() || command_link.is_err() {
        msg.channel_id
            .send_message(&ctx.http, |m| m.content("Need a name and a link."))
            .await?;
    } else {
        let data = ctx.data.read().await;
        let db = data.get::<Db>().expect("Expected Db in TypeMap.");
        db.set_learned(&command_name.unwrap(), &command_link.unwrap())
            .await?;
    }

    Ok(())
}

#[command]
pub async fn learned(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let db = data.get::<Db>().unwrap();

    let commands = db.get_learned_list().await?;

    let mut content = String::from("> List of learned commands:\n");
    for command in commands {
        let line = format!("> - {command}\n");
        content.push_str(&line);
    }

    msg.channel_id
        .send_message(&ctx.http, |m| m.content(content))
        .await?;

    Ok(())
}
