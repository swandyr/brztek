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
    db::{from_i64, Db},
    levels::xp_for_level,
    rank_card::gen_card,
    top_ten_card::gen_top_ten_card,
};

#[command]
pub async fn rank(ctx: &Context, msg: &Message) -> CommandResult {
    let user_id = msg.author.id.0;
    // let channel_id = msg.channel_id.0;

    if let Some(db) = ctx.data.read().await.get::<Db>() {
        let user_level = db.get_user(user_id).await?;

        // Generate a rank card and attach it to a message
        let username = format!("{}#{}", msg.author.name, msg.author.discriminator);
        let avatar_url = msg.author.avatar_url();
        let xp_next_level = xp_for_level(user_level.level + 1);
        let user_http = ctx.http.get_user(user_id).await;
        let banner_colour = if let Ok(user) = user_http {
            user.accent_colour.unwrap_or(Colour::LIGHTER_GREY).tuple()
        } else {
            Colour::LIGHTER_GREY.tuple()
        };

        // Generate a rank card that is saved with name "rank.png"
        // Send an embed message if the function returns an error
        if gen_card(
            &username,
            avatar_url,
            banner_colour,
            user_level.level,
            user_level.xp,
            xp_next_level,
        )
        .await
        .is_err()
        {
            msg.channel_id
                .send_message(&ctx.http, |m| {
                    m.embed(|e| {
                        let name = msg.author.name.clone();
                        let thumbnail = msg.author.avatar_url().unwrap_or_default();
                        let value = format!(
                            "Xp: {}\nLevel:{}\nMessages:{}",
                            user_level.xp, user_level.level, user_level.messages
                        );

                        e.title("Rank")
                            .field(name, value, false)
                            .thumbnail(thumbnail)
                    })
                })
                .await?;
        }

        msg.channel_id
            .send_message(&ctx.http, |m| {
                let file = AttachmentType::from("rank.png");
                m.add_file(file)
            })
            .await?;
    }

    Ok(())
}

#[command]
pub async fn top(ctx: &Context, msg: &Message) -> CommandResult {
    if let Some(db) = ctx.data.read().await.get::<Db>() {
        let mut all_users_id = db.get_all_users().await?;
        all_users_id.sort_by(|a, b| b.xp.cmp(&a.xp));
        let top_x = 10;

        let mut top_users = vec![];
        for (i, user) in all_users_id.iter().enumerate() {
            if i == top_x {
                break;
            }

            let name = UserId::from(from_i64(user.user_id))
                .to_user(&ctx.http)
                .await?
                .name;
            let rank = i as i64 + 1;
            let next_xp = xp_for_level(user.level + 1);
            let user_tup = (name, rank, user.level, user.xp, next_xp);
            top_users.push(user_tup);
        }

        gen_top_ten_card(&top_users).await?;

        msg.channel_id
            .send_message(&ctx.http, |m| {
                let file = AttachmentType::from("top_ten.png");
                m.add_file(file)
            })
            .await?;
    }

    Ok(())
}

#[command]
pub async fn delete_ranks(ctx: &Context, msg: &Message) -> CommandResult {
    if let Some(db) = ctx.data.read().await.get::<Db>() {
        debug!("Delete rows in table 'edn_ranks'");
        db.delete_table().await?;

        msg.channel_id.say(&ctx.http, "All xp dropped to 0").await?;
    }

    Ok(())
}
