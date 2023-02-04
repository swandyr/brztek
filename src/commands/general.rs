use log::{error, info};
use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::{
        channel::Message,
        prelude::{Mention, UserId},
    },
    prelude::*,
};

#[command]
pub async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id
        .send_message(&ctx.http, |m| m.content("Pong!"))
        .await?;
    Ok(())
}

#[command]
pub async fn hello(ctx: &Context, msg: &Message) -> CommandResult {
    let user_id = msg.author.id.0;
    let _channel_id = msg.channel_id.0;

    info!("user_id = {user_id}");
    let user = UserId::from(user_id).to_user(&ctx).await;
    let _user = match user {
        Ok(user) => {
            info!("Found user: {}", user.name);
            Some(user)
        }
        Err(why) => {
            error!("Error: {why}");
            None
        }
    };

    if let Some(user) = _user {
        msg.channel_id
            .send_message(&ctx.http, |m| {
                let mention = Mention::from(user.id);
                let message = format!("Hey, {mention}!");
                m.content(&message)
            })
            .await?;
    }

    Ok(())
}

#[command]
pub async fn say(ctx: &Context, msg: &Message) -> CommandResult {
    let content = &msg.content.replace("!say ", "");
    msg.reply(ctx, content).await?;
    Ok(())
}
