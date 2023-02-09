use serenity::{
    framework::standard::{macros::hook, CommandResult},
    model::channel::Message,
    prelude::*,
};
use tracing::error;

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
pub async fn unknown_command(_ctx: &Context, _msg: &Message, unknown_command_name: &str) {
    println!("Could not find command named '{unknown_command_name}'");
}
