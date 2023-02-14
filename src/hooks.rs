use serenity::{
    framework::standard::{macros::hook, CommandResult},
    model::channel::Message,
    prelude::*,
};
use tracing::error;

use crate::utils::db::Db;

#[hook]
pub async fn after(
    _ctx: &Context,
    _msg: &Message,
    command_name: &str,
    command_result: CommandResult,
) {
    if let Err(why) = command_result {
        error!("Command '{command_name}' returned error '{why:?}'.");
    }
}

#[hook]
pub async fn unknown_command(ctx: &Context, msg: &Message, unknown_command_name: &str) {
    let data = ctx.data.read().await;
    let db = data.get::<Db>().expect("Expected Db in TypeMap");

    let queried = db
        .get_command(unknown_command_name)
        .await
        .expect("Query learned_command return error.");

    match queried {
        Some(link) => {
            msg.channel_id
                .send_message(&ctx.http, |m| m.content(link))
                .await
                .expect("Error with send learned command link");
        }
        None => {
            let content = format!("Could not find command named '{unknown_command_name}'");
            msg.reply(&ctx.http, content)
                .await
                .expect("Error with hook 'unknown command'");
        }
    }
}
