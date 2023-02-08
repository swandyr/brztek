use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::{
        channel::Message,
        prelude::{Mention, UserId},
    },
    prelude::*,
};
use std::env;
use tracing::{error, info};

use crate::utils::config::Config;

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
    // let channel_id = msg.channel_id.0;

    info!("user_id = {user_id}");
    let get_user = UserId::from(user_id).to_user(&ctx).await;
    match get_user {
        Ok(user) => {
            msg.channel_id
                .send_message(&ctx.http, |m| {
                    let mention = Mention::from(user.id);
                    let message = format!("Hey, {mention}!");
                    m.content(&message)
                })
                .await?;
        }
        Err(why) => {
            error!("Error: {why}");
        }
    };

    Ok(())
}

#[command]
pub async fn welcome(ctx: &Context, msg: &Message) -> CommandResult {
    use rand::prelude::*;
    use serenity::constants::JOIN_MESSAGES;

    let len = JOIN_MESSAGES.len();
    let index = thread_rng().gen_range(0..len);
    let message = JOIN_MESSAGES.get(index).unwrap();
    let mention = Mention::User(msg.author.id);
    let message = message.replace("$user", &format!("{mention}"));

    if let Ok(chan) = env::var("GENERAL_CHANNEL_ID") {
        if let Ok(id) = chan.parse::<u64>() {
            ctx.cache
                .guild_channel(id)
                .unwrap()
                .send_message(&ctx, |m| m.content(message))
                .await
                .unwrap();
        } else {
            error!("Unable to parse GENERAL_CHANNEL_ID; check var in .env file.");
        }
    } else {
        error!("Unable to find GENERAL_CHANNEL_ID; check var in .env file.");
    };

    Ok(())
}
