use log::{debug, error, info};

use serenity::framework::standard::macros::command;
use serenity::framework::standard::CommandResult;
use serenity::model::channel::Message;
use serenity::model::prelude::UserId;
use serenity::model::user::User;
use serenity::prelude::*;

use crate::Db;

#[command]
pub async fn rank(ctx: &Context, msg: &Message) -> CommandResult {
    let user_id = msg.author.id.0;
    let _channel_id = msg.channel_id.0;

    let db = ctx.data.read().await.get::<Db>().unwrap().clone();

    let user_xp = db.get_user_xp(user_id).await.unwrap();
    let _message = format!("{} has {} xp", msg.author.name, user_xp);

    let _ = msg
        .channel_id
        .send_message(ctx, |m| {
            m.embed(|e| {
                let name = msg.author.name.clone();
                let thumbnail = msg.author.avatar_url().unwrap_or_default();
                let value = user_xp;

                e.title("Rank")
                    .field(name, value, false)
                    .thumbnail(thumbnail)
            })
        })
        .await?;

    Ok(())
}

#[command]
pub async fn top(ctx: &Context, msg: &Message) -> CommandResult {
    let db = ctx.data.read().await.get::<Db>().unwrap().clone();

    let mut all_users_id = db.get_all_users_xp().await?;
    all_users_id.sort_by(|a, b| b.1.cmp(&a.1));

    let mut field_names = String::new();
    let mut field_xp = String::new();
    let mut _field_levels = String::new();
    let mut field_ranks = String::new();
    for (i, (id, xp, _lvl)) in all_users_id.iter().enumerate() {
        let user = UserId::from(*id).to_user(&ctx).await.unwrap_or_default();
        field_names.push_str(&format!("{}\n", user.name));
        field_xp.push_str(&format!("{}\n", xp));
        _field_levels.push_str(&format!("{}\n", _lvl));
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

    let _ = msg
        .channel_id
        .send_message(ctx, |m| {
            m.embed(|e| {
                if let Some(icon) = thumbnail {
                    e.title("Top Spammers")
                        .thumbnail(icon)
                        .field("Rank", field_ranks, true)
                        .field("Name", field_names, true)
                        .field("Xp", field_xp, true)
                } else {
                    e.title("Top Spammers")
                        .field("Rank", field_ranks, true)
                        .field("Name", field_names, true)
                        .field("Xp", field_xp, true)
                }
            })
        })
        .await?;

    Ok(())
}

#[command]
pub async fn delete_ranks(ctx: &Context, msg: &Message) -> CommandResult {
    let db = ctx.data.read().await.get::<Db>().unwrap().clone();
    debug!("Delete rows in table 'edn_ranks'");
    db.delete_table().await?;

    msg.channel_id.say(ctx, "All xp dropped to 0").await?;

    Ok(())
}
