use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::{
        channel::Message,
        prelude::{Mention, UserId},
    },
    prelude::*,
};
use tracing::{error, info};

#[command]
pub async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id
        .send_message(&ctx.http, |m| m.content("Pong!"))
        .await?;
    Ok(())
}
