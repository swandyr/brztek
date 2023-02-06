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
};

#[command]
pub async fn rank(ctx: &Context, msg: &Message) -> CommandResult {
    let user_id = msg.author.id.0;
    // let channel_id = msg.channel_id.0;

    if let Some(db) = ctx.data.read().await.get::<Db>() {
        let user_level = db.get_user(user_id).await?;

        // Send an embedded message
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

        // Generate a rank card and attach it to a message
        let username = format!("{}#{}", msg.author.name, msg.author.discriminator);
        let avatar_url = msg.author.avatar_url();
        let xp_next_level = xp_for_level(user_level.level);
        let user_http = ctx.http.get_user(user_id).await;
        let banner_colour = if let Ok(user) = user_http {
            user.accent_colour.unwrap_or(Colour::LIGHTER_GREY).tuple()
        } else {
            Colour::LIGHTER_GREY.tuple()
        };

        gen_card(
            &username,
            avatar_url,
            banner_colour,
            user_level.level,
            user_level.xp,
            xp_next_level,
        )
        .await?;
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

        let mut field_names = String::new();
        let mut field_xp = String::new();
        let mut field_levels = String::new();
        let mut field_ranks = String::new();

        for (i, user_level) in all_users_id.iter().enumerate() {
            let user = UserId::from(from_i64(user_level.user_id))
                .to_user(&ctx)
                .await
                .unwrap_or_default();
            field_names.push_str(&format!("{}\n", user.name));
            field_xp.push_str(&format!("{}\n", user_level.xp));
            field_levels.push_str(&format!("{}\n", user_level.level));
            field_ranks.push_str(&format!("{}\n", i + 1));
        }

        // let thumbnail = match msg.guild(ctx) {
        //     Some(guild) => guild.icon,
        //     None => {
        //         error!("No icon found");
        //         None
        //     }
        // };

        let thumbnail = ctx.cache.current_user().avatar;

        info!("Bot icon: {thumbnail:?}");

        msg.channel_id
            .send_message(&ctx.http, |m| {
                m.embed(|e| {
                    if let Some(icon) = thumbnail {
                        e.title("Top Spammers")
                            .thumbnail(icon)
                            .field("Rank", field_ranks, true)
                            .field("Name", field_names, true)
                            .field("Xp", field_xp, true)
                            .field("Level", field_levels, true)
                    } else {
                        e.title("Top Spammers")
                            .field("Rank", field_ranks, true)
                            .field("Name", field_names, true)
                            .field("Xp", field_xp, true)
                            .field("Level", field_levels, true)
                    }
                })
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
