use poise::serenity_prelude as serenity;
use std::time::Instant;
use tracing::{debug, info, instrument};

use super::queries;
use crate::{Data, Db, Error};

#[instrument(skip_all)]
pub async fn add_xp(
    ctx: &serenity::Context,
    user_data: &Data,
    guild_id: &serenity::GuildId,
    channel_id: &serenity::ChannelId,
    user_id: &serenity::UserId,
) -> Result<(), Error> {
    let db = &user_data.db;
    let mut user = queries::get_user(db, user_id.get(), guild_id.get()).await?;

    // User gain xp if the time defined by spam_delay parameter in xp_settings
    // has passed since his last message
    let has_gained_xp = user.gain_xp_if_not_spam();

    // Update user in database with new xp and level
    if has_gained_xp {
        info!("User has gained XP");
        // Increment level of the user if enough xp, then send a chat message
        if user.has_level_up() {
            info!("User has levelled up");
            let mention = serenity::Mention::from(*user_id);
            let message = format!("Level Up, {mention}!");
            channel_id
                .send_message(&ctx.http, serenity::CreateMessage::new().content(&message))
                .await?;
        }

        let t_0 = Instant::now();
        queries::update_user(db, &user, guild_id.get()).await?;
        debug!("Updated user : {user:#?}");
        debug!("update_user finished in {} µs", t_0.elapsed().as_micros());

        let t_1 = Instant::now();
        // Recalculate ranking of the user in the guild
        update_users_ranks(db, guild_id.get()).await?;
        debug!("Updated ranks");
        debug!(
            "update_users_ranks finished in {} µs",
            t_1.elapsed().as_micros()
        );
    }

    Ok(())
}

#[allow(clippy::cast_possible_wrap)]
#[instrument(skip_all)]
async fn update_users_ranks(db: &Db, guild_id: u64) -> Result<(), Error> {
    // Get a Vec of all users in database
    let mut all_users = queries::get_all_users(db, guild_id).await?;

    // Sort user by descendant xp
    all_users.sort_by(|a, b| b.xp.cmp(&a.xp));

    let mut rank_has_changed = vec![];
    for (i, user) in &mut all_users.iter_mut().enumerate() {
        if user.rank != i as i64 + 1 {
            user.rank = i as i64 + 1;
            rank_has_changed.push(*user);
        }
    }

    if !rank_has_changed.is_empty() {
        queries::update_ranks(db, rank_has_changed, guild_id).await?;
    }

    Ok(())
}
