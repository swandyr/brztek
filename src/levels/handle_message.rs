use poise::serenity_prelude as serenity;
use std::time::Instant;
use tracing::{debug, info, instrument};

use super::queries;
use crate::{Data, Db};

#[instrument(skip_all)]
pub async fn add_xp(
    ctx: &serenity::Context,
    user_data: &Data,
    guild_id: &serenity::GuildId,
    channel_id: &serenity::ChannelId,
    user_id: &serenity::UserId,
) -> anyhow::Result<()> {
    let db = &user_data.db;
    let mut user = queries::get_user(db, user_id.0, guild_id.0).await?;

    // User gain xp if the time defined by spam_delay parameter in xp_settings
    // has passed since his last message
    let has_gained_xp = user.gain_xp_if_not_spam();

    // Update user in database with new xp and level
    if has_gained_xp {
        // Increment level of the user if enough xp, then send a chat message
        if user.has_level_up() {
            channel_id
                .send_message(&ctx.http, |m| {
                    let mention = serenity::Mention::from(*user_id);
                    let message = format!("Level Up, {mention}!");
                    m.content(&message)
                })
                .await?;
        }

        queries::update_user(db, &user, guild_id.0).await?;
        debug!("Update user : {user:#?}");

        // Recalculate ranking of the user in the guild
        update_users_ranks(db, guild_id.0).await?;
    }

    Ok(())
}

#[instrument(skip_all)]
async fn update_users_ranks(db: &Db, guild_id: u64) -> anyhow::Result<()> {
    let t_0 = Instant::now();

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

    info!("Updated all ranks in : {} µs", t_0.elapsed().as_micros());

    Ok(())
}
