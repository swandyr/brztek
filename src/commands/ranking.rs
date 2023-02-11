use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::{
        channel::Message,
        prelude::{AttachmentType, UserId},
    },
    prelude::*,
    utils::Colour,
};
use tracing::{debug, info};

use crate::utils::{
    db::Db,
    levels::{rank_card::gen_card, top_ten_card::gen_top_ten_card},
};

#[command]
#[description = "Print your level stats"]
pub async fn rank(ctx: &Context, msg: &Message) -> CommandResult {
    let user_id = msg.author.id.0;
    // let channel_id = msg.channel_id.0;

    let data = ctx.data.read().await;
    let db = data.get::<Db>().expect("Expected Db in TypeMap.");

    // Get user from database
    let user_level = db.get_user(user_id).await?;

    // Generate a rank card and attach it to a message
    let username = format!("{}#{}", msg.author.name, msg.author.discriminator);
    let avatar_url = msg.author.avatar_url();
    let user_http = ctx.http.get_user(user_id).await?;
    let banner_colour = user_http
        .accent_colour
        .unwrap_or(Colour::LIGHTER_GREY)
        .tuple();

    // Generate an image that is saved with name "rank.png"
    gen_card(
        &username,
        avatar_url,
        banner_colour,
        user_level.level,
        user_level.xp,
    )
    .await?;

    // Send generated "rank.png" file
    msg.channel_id
        .send_message(&ctx.http, |m| {
            let file = AttachmentType::from("rank.png");
            m.add_file(file)
        })
        .await?;

    // msg.channel_id
    //         .send_message(&ctx.http, |m| {
    //             m.embed(|e| {
    //                 let name = msg.author.name.clone();
    //                 let thumbnail = msg.author.avatar_url().unwrap_or_default();
    //                 let value = format!(
    //                     "Xp: {}\nLevel:{}\nMessages:{}",
    //                     user_level.xp, user_level.level, user_level.messages
    //                 );

    //                 e.title("Rank")
    //                     .field(name, value, false)
    //                     .thumbnail(thumbnail)
    //             })
    //         })
    //         .await?;

    Ok(())
}

#[command]
#[description = "Show the 10 most active users"]
pub async fn top(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let db = data.get::<Db>().expect("Expected Db in TypeMap.");

    // Get a vec of all users in database
    let mut all_users_id = db.get_all_users().await?;

    // Sort users by descendant xp
    all_users_id.sort_by(|a, b| b.xp.cmp(&a.xp));

    // Number of users to keep
    let top_x = 10;

    let mut top_users = vec![];
    for (i, user) in all_users_id.iter().enumerate() {
        // Break when enough users are collected
        if i == top_x {
            break;
        }

        let name = UserId::from(user.user_id).to_user(&ctx.http).await?.name;
        let rank = i as i64 + 1;
        let user_tup = (name, rank, user.level, user.xp);
        top_users.push(user_tup);
    }

    // Generate an image that is saved with name "top_ten.png"
    gen_top_ten_card(&top_users).await?;

    // Send generated "top_ten.png" file
    msg.channel_id
        .send_message(&ctx.http, |m| {
            let file = AttachmentType::from("top_ten.png");
            m.add_file(file)
        })
        .await?;

    Ok(())
}
