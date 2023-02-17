use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::{
        channel::Message,
        prelude::{AttachmentType, UserId},
    },
    prelude::*,
    utils::Colour,
};
use std::time::Instant;
use tracing::info;

use crate::levels::{rank_card::gen_card, top_ten_card::gen_top_ten_card};
use crate::utils::db::Db;

#[command]
#[description = "Print your level stats"]
pub async fn rank(ctx: &Context, msg: &Message) -> CommandResult {
    let t_0 = Instant::now();

    let user_id = msg.author.id.0;

    let data = ctx.data.read().await;
    let db = data.get::<Db>().expect("Expected Db in TypeMap.");

    // Ensure the command was sent from a guild channel
    let guild_id = if let Some(id) = msg.guild_id {
        id.0
    } else {
        msg.channel_id
            .send_message(&ctx.http, |m| m.content("No guild id found"))
            .await?;
        return Ok(());
    };

    // Get user from database
    let user_level = db.get_user(user_id, guild_id).await?;

    // Generate a rank card and attach it to a message
    let username = format!("{}#{}", msg.author.name, msg.author.discriminator);
    let avatar_url = msg.author.avatar_url();
    let user_http = ctx.http.get_user(user_id).await?;
    let banner_colour = user_http
        .accent_colour
        .unwrap_or(Colour::LIGHTER_GREY)
        .tuple();

    // Generate an image that is saved with name "rank.png"
    let t_1 = Instant::now();
    gen_card(
        &username,
        avatar_url,
        banner_colour,
        user_level.level,
        user_level.rank,
        user_level.xp,
    )
    .await?;
    info!("Rank_card generated in : {} µs", t_1.elapsed().as_micros());

    let t_1 = Instant::now();
    // Send generated "rank.png" file
    msg.channel_id
        .send_message(&ctx.http, |m| {
            let file = AttachmentType::from("rank.png");
            m.add_file(file)
        })
        .await?;
    info!("rank_card sent in : {} µs", t_1.elapsed().as_micros());

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

    info!(
        "Command !rank processed in : {} µs",
        t_0.elapsed().as_micros()
    );

    Ok(())
}

#[command]
#[description = "Show the 10 most active users"]
pub async fn top(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let data = ctx.data.read().await;
    let db = data.get::<Db>().expect("Expected Db in TypeMap.");

    // Number of users to keep
    let top_x = if let Ok(num) = args.single::<usize>() {
        num
    } else {
        10
    };

    // Ensure the command was sent from a guild channel
    let guild_id = if let Some(id) = msg.guild_id {
        id.0
    } else {
        msg.channel_id
            .send_message(&ctx.http, |m| m.content("No guild id found"))
            .await?;
        return Ok(());
    };

    let guild_name = msg.guild_field(ctx, |guild| guild.name.to_owned()).unwrap();

    // Get a vec of all users in database
    let mut all_users_id = db.get_all_users(guild_id).await?;

    // Sort users by descendant xp
    all_users_id.sort_by(|a, b| a.rank.cmp(&b.rank));

    let mut top_users = vec![];
    for user in all_users_id.iter().take(top_x) {
        let name = UserId::from(user.user_id).to_user(&ctx.http).await?.name;
        let user_tup = (name, user.rank, user.level, user.xp);
        top_users.push(user_tup);
    }

    // Generate an image that is saved with name "top_ten.png"
    gen_top_ten_card(&top_users, &guild_name).await?;

    // Send generated "top_ten.png" file
    msg.channel_id
        .send_message(&ctx.http, |m| {
            let file = AttachmentType::from("top_ten.png");
            m.add_file(file)
        })
        .await?;

    Ok(())
}
