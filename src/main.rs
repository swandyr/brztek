mod commands;
mod levels;
mod utils;

use poise::serenity_prelude::{self as serenity, Mentionable};
use rand::{prelude::thread_rng, Rng};
use std::{env, time::Instant};
use tracing::{error, info, instrument};

use utils::db::Db;

type Error = Box<dyn std::error::Error + Send + Sync>;
// type Context<'a> = poise::Context<'a, Data, Error>;

const PREFIX: &str = "$";

/// Store shared data
#[derive(Debug)]
pub struct Data {
    pub db: std::sync::Arc<Db>,
}

// ------------------------------------- Event handler -----------------------------------------

#[instrument(skip(ctx, _framework))]
async fn event_event_handler(
    ctx: &serenity::Context,
    event: &poise::Event<'_>,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    user_data: &Data,
) -> Result<(), Error> {
    match event {
        poise::Event::Ready { data_about_bot } => {
            info!("{} is connected.", data_about_bot.user.name);
        }

        poise::Event::CacheReady { guilds } => {
            let db = &user_data.db;

            for guild in guilds {
                let guild_id = guild.0;
                db.create_config_entry(guild_id).await?;
            }
        }

        poise::Event::Message { new_message } => {
            let t_0 = Instant::now();

            // Do not handle message from bot users
            if new_message.author.bot {
                return Ok(());
            }

            // Ensure the command was sent from a guild channel
            let guild_id = if let Some(id) = new_message.guild_id {
                id
            } else {
                return Ok(());
            };

            let user_id = new_message.author.id;
            let channel_id = new_message.channel_id;

            levels::handle_message_xp(ctx, user_data, &guild_id, &channel_id, &user_id).await?;

            info!("Message processed in: {} µs", t_0.elapsed().as_micros());
        }

        //? Discord already do this
        poise::Event::GuildMemberAddition { new_member } => {
            let join_messages = serenity::constants::JOIN_MESSAGES;
            let index = thread_rng().gen_range(0..join_messages.len());
            let mention = new_member.mention();
            let content = join_messages
                .get(index)
                .unwrap_or(&"Welcome $user")
                .replace("$user", &format!("{mention}"));
            let guild_id = new_member.guild_id.0;

            let channel_id = user_data.db.get_pub_channel_id(guild_id).await?;
            if let Some(id) = channel_id {
                ctx.cache
                    .guild_channel(id)
                    .unwrap()
                    .send_message(&ctx.http, |m| m.content(content))
                    .await?;
            }
        }

        poise::Event::GuildMemberRemoval {
            guild_id,
            user,
            member_data_if_available: _,
        } => {
            let username = format!("{}{}", user.name, user.discriminator);
            let content = format!("RIP **{username}**, you'll be missed maybe");
            let guild_id = guild_id.0;

            let channel_id = user_data.db.get_pub_channel_id(guild_id).await?;

            if let Some(id) = channel_id {
                ctx.cache
                    .guild_channel(id)
                    .unwrap()
                    .send_message(&ctx.http, |m| m.content(content))
                    .await?;
            }
        }
        _ => {}
    }

    Ok(())
}

// -------------------------------------- Error handling ----------------------------------

#[instrument]
async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    match error {
        poise::FrameworkError::Setup {
            error,
            framework: _,
            data_about_bot,
            ctx: _,
        } => {
            error!("Error during setup: {error:?}\ndata_about_bot: {data_about_bot:#?}");
        }

        poise::FrameworkError::EventHandler {
            error,
            ctx: _,
            event,
            framework: _,
        } => {
            error!("Error while handling event {event:?}: {error:?}");
        }

        poise::FrameworkError::UnknownCommand {
            ctx,
            msg,
            msg_content,
            framework,
            ..
        } => {
            // Check in database if it's a learned command
            let db = &framework.user_data.db;
            let guild_id = msg.guild_id.unwrap().0;

            let queried = db
                .get_learned(msg_content, guild_id)
                .await
                .expect("Query learned_command returned with error");
            if let Some(link) = queried {
                msg.channel_id
                    .send_message(&ctx.http, |m| m.content(link))
                    .await
                    .expect("Error sending learned command link");
            } else {
                msg.channel_id
                    .send_message(&ctx.http, |m| m.content("https://tenor.com/view/kaamelott-perceval-cest-pas-faux-not-false-gif-17161490"))
                    .await
                    .unwrap();
            }
        }

        poise::FrameworkError::MissingUserPermissions {
            missing_permissions,
            ctx,
        } => {
            info!(
                "{} used command {} but misses permissions: {}",
                ctx.author().name,
                ctx.command().name,
                missing_permissions.unwrap()
            );
            ctx.channel_id()
                .send_message(&ctx, |m| {
                    m.content(
                "https://tenor.com/view/jurrasic-park-samuel-l-jackson-magic-word-you-didnt-say-the-magic-work-gif-3556977",
            )
                })
                .await
                .unwrap();
        }

        poise::FrameworkError::MissingBotPermissions {
            missing_permissions,
            ctx,
        } => {
            error!(
                "Bot misses permissions: {} for command {}",
                missing_permissions,
                ctx.command().name
            );
        }

        poise::FrameworkError::GuildOnly { ctx } => {
            ctx.say("This does not work outside a guild.")
                .await
                .unwrap();
        }

        poise::FrameworkError::Command { error, ctx: _ } => {
            error!("Error in command: {}", error);
        }

        error => {
            error!("Unhandled error on command: {error}")
        }
    }
}

// ----------------------------------------- Main -----------------------------------------

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenvy::dotenv().expect("Failed to load .env file");
    tracing_subscriber::fmt().init();

    let token = env::var("DISCORD_TOKEN").expect("token needed");
    //? Intents are still a mystery to me
    let intents = serenity::GatewayIntents::non_privileged()
        | serenity::GatewayIntents::MESSAGE_CONTENT
        | serenity::GatewayIntents::GUILD_MEMBERS;

    let db_url = env::var("DATABASE_URL").expect("database path not found");
    let db = Db::new(&db_url).await;
    db.run_migrations().await.expect("Unable to run migrations");

    let options = poise::FrameworkOptions {
        commands: vec![
            commands::register(),
            commands::help(),
            commands::general::ping(),
            commands::general::learn(),
            commands::general::learned(),
            commands::general::bigrig(),
            commands::general::set_color(),
            commands::general::yt(),
            commands::levels::rank(),
            commands::levels::top(),
            commands::admin::admin(),
            commands::admin::import_mee6_levels(),
        ],
        event_handler: |ctx, event, framework, user_data| {
            Box::pin(event_event_handler(ctx, event, framework, user_data))
        },
        prefix_options: poise::PrefixFrameworkOptions {
            prefix: Some(PREFIX.into()),
            case_insensitive_commands: true,
            ..Default::default()
        },
        pre_command: |ctx| {
            Box::pin(async move {
                info!("Executing command {}", ctx.command().qualified_name);
            })
        },
        on_error: |error| Box::pin(on_error(error)),
        ..Default::default()
    };

    // The Framework builder will automatically retrieve the bot owner and application ID via the
    // passed token, so that information need not be passed here
    if let Err(why) = poise::Framework::builder()
        .token(token)
        .intents(intents)
        .options(options)
        .setup(|_ctx, _data_about, _framework| {
            Box::pin(async move {
                Ok(Data {
                    db: std::sync::Arc::new(db),
                })
            })
        })
        .run()
        .await
    {
        error!("Client returned with error: {why}");
    }

    Ok(())
}
