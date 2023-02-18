mod levels;
mod poise_commands;
mod utils;

use poise::serenity_prelude::{self as serenity, Mentionable};
use rand::{prelude::thread_rng, Rng};
use std::{env, time::Instant};
use tracing::{debug, error, info};

use utils::db::Db;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

const PREFIX: &str = "$";

/// Store database accessor
pub struct Data {
    pub db: std::sync::Arc<Db>,
}

// ------------------------------------- Event handler -----------------------------------------

async fn event_event_handler(
    ctx: &serenity::Context,
    event: &poise::Event<'_>,
    framework: poise::FrameworkContext<'_, Data, Error>,
    user_data: &Data,
) -> Result<(), Error> {
    match event {
        poise::Event::Ready { data_about_bot } => {
            info!("{} is connected.", data_about_bot.user.name);
        }
        poise::Event::Message { new_message } => {
            let t_0 = Instant::now();

            // Ensure the command was sent from a guild channel
            let guild_id = if let Some(id) = new_message.guild_id {
                id
            } else {
                return Ok(());
            };

            // poise does

            let user_id = new_message.author.id;
            let channel_id = new_message.channel_id;

            levels::handle_message_xp(ctx, user_data, &guild_id, &channel_id, &user_id).await?;

            info!("Message processed in: {} µs", t_0.elapsed().as_micros());
        }
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
            member_data_if_available,
        } => {
            let username = format!("{}{}", user.name, user.discriminator);
            let content = format!("RIP **{username}**, you'll be missed");
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

async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    match error {
        poise::FrameworkError::UnknownCommand {
            ctx,
            msg,
            msg_content,
            framework,
            ..
        } => {
            // Check in database if it's a learned command
            let db = &framework.user_data.db;

            let queried = db
                .get_learned(msg_content)
                .await
                .expect("Query learned_command returned with error");
            if let Some(link) = queried {
                msg.channel_id
                    .send_message(&ctx.http, |m| m.content(link))
                    .await
                    .expect("Error sending learned command link");
            } else {
                msg.channel_id
                    .send_message(&ctx.http, |m| m.content("This is not a valid command."))
                    .await
                    .unwrap();
            }
        }
        error => error!("Got some error: {error}"),
    }
}

// ----------------------------------------- Main -----------------------------------------

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenvy::dotenv().expect("Failed to load .env file");
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_target(false)
        .init();

    let token = env::var("DISCORD_TOKEN").expect("token needed");
    let intents = serenity::GatewayIntents::non_privileged()
        | serenity::GatewayIntents::MESSAGE_CONTENT
        | serenity::GatewayIntents::GUILDS
        | serenity::GatewayIntents::GUILD_MEMBERS;

    let db_url = env::var("DATABASE_URL").expect("database path not found");
    let db = Db::new(&db_url).await;
    db.run_migrations().await.expect("Unable to run migrations");
    // Set config entry if not exists

    // let mut config = Config::load().unwrap_or_else(|err| {
    //     error!("Can't read config file: {err}");
    //     Config::default()
    // });

    let options = poise::FrameworkOptions {
        commands: vec![
            poise_commands::register(),
            poise_commands::help(),
            poise_commands::general::ping(),
            poise_commands::general::learn(),
            poise_commands::general::learned(),
            poise_commands::levels::rank(),
            poise_commands::levels::top(),
            poise_commands::admin::admin(),
        ],
        event_handler: |ctx, event, framework, user_data| {
            Box::pin(event_event_handler(ctx, event, framework, user_data))
        },
        prefix_options: poise::PrefixFrameworkOptions {
            prefix: Some(PREFIX.into()),
            ..Default::default()
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
