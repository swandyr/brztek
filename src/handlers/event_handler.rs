mod member;
mod message;

use poise::serenity_prelude::{self as serenity, parse_message_url};
use std::{mem, sync::Arc, time::Instant};
use tracing::{debug, error, info, instrument, trace};

use crate::{database, youtube, Context, Data, Error};

#[instrument(skip_all)]
pub async fn on_event(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    framework: poise::FrameworkContext<'_, Data, Error>,
    user_data: &Data,
) -> Result<(), Error> {
    match event {
        serenity::FullEvent::Ready { data_about_bot } => {
            info!("{} is connected.", data_about_bot.user.name);
        }

        serenity::FullEvent::CacheReady { guilds } => {
            let db = &user_data.db;

            for guild in guilds {
                let guild_id = guild.get();
                database::add_guild(db, guild_id).await?;
                let permissions = guild
                    .member(ctx, framework.bot_id)
                    .await?
                    .permissions(ctx)?;
                info!("Connected to guild: {:?} (id {})", guild.name(ctx), guild);
                info!("Permissions: {:#?}", permissions);
            }

            // Starts the listener in a separate thread
            let db_c = Arc::clone(db);
            let serenity_ctx = ctx.clone();
            let listener = Arc::clone(&user_data.hook_listener);
            std::thread::spawn(move || {
                if let Err(e) = youtube::listen_loop(serenity_ctx, db_c, listener) {
                    error!("in listen_loop: {e}");
                }
            });

            // Starts the expiration checker
            let db_c = Arc::clone(db);
            let listener = Arc::clone(&user_data.hook_listener);
            std::thread::spawn(move || {
                if let Err(e) = youtube::expiration_check_timer(listener, db_c) {
                    error!("in expiration_check_timer: {e}");
                }
            });
        }

        serenity::FullEvent::Message { new_message } => {
            trace!("New message received: author: {}", new_message.author.name);
            let t_0 = Instant::now();

            // Do not handle message from bot users
            if new_message.author.bot {
                trace!("Author is a bot, ignored");
                return Ok(());
            }

            // Ensure the command was sent from a guild channel
            if new_message.guild_id.is_none() {
                trace!("Message is not from a guild, ignored");
                return Ok(());
            };

            message::message_handler(new_message, ctx, user_data).await?;

            debug!("Message processed in: {} µs", t_0.elapsed().as_micros());
        }

        //? Discord already do this
        serenity::FullEvent::GuildMemberAddition { new_member } => {
            info!("New member added: {}", new_member.user.name);
            member::member_addition_handler(new_member, ctx).await?;
        }

        serenity::FullEvent::GuildMemberRemoval {
            guild_id,
            user,
            member_data_if_available: _,
        } => {
            info!("Member removed: {}", user.name);
            member::member_removal_handler(guild_id, user, ctx).await?;
        }
        _ => {}
    }

    Ok(())
}
