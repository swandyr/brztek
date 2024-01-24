#![allow(
    clippy::unused_async,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_lossless,
    unused
)]

mod builtins;
mod clearurl;
mod commands;
mod config;
mod db;
mod handlers;

use commands::youtube;
use poise::{
    serenity_prelude::{self as serenity, UserId},
    CreateReply,
};
use std::{
    collections::HashMap,
    env,
    sync::{mpsc, Arc, Mutex},
    time::Instant,
};
use tracing::{debug, error, info, instrument, trace, warn};
use tracing_subscriber::{fmt, layer::SubscriberExt, EnvFilter};

use brzthook::prelude::*;

use config::Config;
use db::Db;

pub(crate) type Error = Box<dyn std::error::Error + Send + Sync>;
pub(crate) type Context<'a> = poise::Context<'a, Data, Error>;

const PREFIX: &str = "$";

/// Store shared data
pub struct Data {
    pub config: Arc<Config>,
    pub db: Arc<Db>,
    pub roulette_map: Arc<Mutex<HashMap<UserId, (u8, i64)>>>,
    pub hook_listener: Arc<HookListener>,
}

// ---------------------------------------- Main -----------------------------------------

#[instrument]
#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenvy::dotenv()?;
    let filter = EnvFilter::from_default_env();

    let file_appender = tracing_appender::rolling::daily("./logs", "log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    let subscriber = tracing_subscriber::registry()
        .with(filter)
        .with(fmt::Layer::new().pretty().with_writer(std::io::stdout))
        .with(
            fmt::Layer::new()
                .compact()
                .with_ansi(false)
                .with_line_number(true)
                .with_writer(non_blocking),
        );
    tracing::subscriber::set_global_default(subscriber)?;

    let token = env::var("DISCORD_TOKEN")?;

    let intents = serenity::GatewayIntents::non_privileged()
        | serenity::GatewayIntents::MESSAGE_CONTENT
        | serenity::GatewayIntents::GUILD_MEMBERS
        | serenity::GatewayIntents::GUILD_PRESENCES;

    let cfg_file = std::fs::read_to_string("config.toml")?;
    let config: Config = toml::from_str(&cfg_file)?;

    //let db_url = env::var("DATABASE_URL")?;
    let db_url = &config.database;
    info!("Connecting to database: {}", &db_url);
    let db = Db::new(db_url).await;
    info!("Connected to database. Running migrations");
    db.run_migrations().await?;

    // Create webhook listener and get receiver
    let hook_listener = HookListener::builder()
        .listener(&config.brzthook.ip_addr, config.brzthook.port)?
        .callback(&config.brzthook.callback)
        .new_only(config.brzthook.new_only)
        .build()?;

    let options = poise::FrameworkOptions {
        commands: vec![
            builtins::help(),
            builtins::register(),
            commands::admin::admin(),
            commands::admin::import_mee6_levels(),
            commands::levels::rank(),
            commands::levels::top(),
            commands::misc::br(),
            commands::misc::clean(),
            commands::misc::learn(),
            commands::misc::learned(),
            commands::misc::ping(),
            commands::misc::setcolor(),
            commands::roulette::rffstar(),
            commands::roulette::roulette(),
            commands::roulette::statroulette(),
            commands::roulette::toproulette(),
            commands::youtube::yt(),
        ],
        event_handler: |ctx, event, framework, user_data| {
            Box::pin(event_handler(ctx, event, framework, user_data))
        },
        prefix_options: poise::PrefixFrameworkOptions {
            prefix: Some(PREFIX.into()),
            case_insensitive_commands: true,
            ..Default::default()
        },
        pre_command: |ctx| {
            let guild_id = ctx.guild().map_or_else(|| 0, |g| g.id.get());
            let guild_name = ctx.guild().map(|g| g.name.clone());
            Box::pin(async move {
                db::increment_cmd(&ctx.data().db, &ctx.command().qualified_name, guild_id)
                    .await
                    .unwrap();
                info!(
                    "Executing command {} in guild {:?}",
                    ctx.command().qualified_name,
                    guild_name
                );
            })
        },
        on_error: |error| Box::pin(on_error(error)),
        ..Default::default()
    };

    // The Framework builder will automatically retrieve the bot owner and application ID via the
    // passed token, so that information need not be passed here
    info!("Starting brztek with intents: {:?}", intents);
    let framework = poise::Framework::builder()
        .options(options)
        .setup(|_ctx, _data_about, _framework| {
            Box::pin(async move {
                Ok(Data {
                    config: Arc::new(config),
                    db: Arc::new(db),
                    roulette_map: Arc::new(Mutex::new(HashMap::new())),
                    hook_listener: Arc::new(hook_listener),
                })
            })
        })
        .build();

    serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await?
        .start()
        .await?;

    Ok(())
}

// ------------------------------------- Event handler -----------------------------------------

#[instrument(skip_all)]
async fn event_handler(
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
                db::add_guild(db, guild_id).await?;
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

            handlers::message_handler(new_message, ctx, user_data).await?;

            debug!("Message processed in: {} µs", t_0.elapsed().as_micros());
        }

        //? Discord already do this
        serenity::FullEvent::GuildMemberAddition { new_member } => {
            info!("New member added: {}", new_member.user.name);
            handlers::member_addition_handler(new_member, ctx).await?;
        }

        serenity::FullEvent::GuildMemberRemoval {
            guild_id,
            user,
            member_data_if_available: _,
        } => {
            info!("Member removed: {}", user.name);
            handlers::member_removal_handler(guild_id, user, ctx).await?;
        }
        _ => {}
    }

    Ok(())
}

// -------------------------------------- Error handling ----------------------------------

#[instrument(skip(error))]
async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    match error {
        poise::FrameworkError::Setup {
            error,
            framework: _,
            data_about_bot,
            ctx: _,
            ..
        } => {
            error!("Error during setup: {error:?}\ndata_about_bot: {data_about_bot:#?}");
        }

        poise::FrameworkError::EventHandler {
            error,
            ctx: _,
            event,
            framework: _,
            ..
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
            // On unknown command, it will first queries the database to check for corresponding
            // entry in the learned table for a user's registered command
            trace!(
                "Unknown command received: {}. Checking for learned commands",
                msg_content
            );
            let db = &framework.user_data.db;
            let guild_id = msg.guild_id.unwrap().get();

            let queried = commands::misc::queries::get_learned(db, msg_content, guild_id)
                .await
                .expect("Query learned_command returned with error");
            if let Some(link) = queried {
                trace!("Learned command found: {}", msg_content);
                msg.channel_id
                    .send_message(&ctx, serenity::CreateMessage::new().content(link))
                    .await
                    .expect("Error sending learned command link");
            } else {
                warn!("Unknown command: {}", msg_content);
                msg.channel_id
                    .send_message(&ctx, serenity::CreateMessage::new().content("https://tenor.com/view/kaamelott-perceval-cest-pas-faux-not-false-gif-17161490"))
                    .await
                    .unwrap();
            }
        }

        poise::FrameworkError::MissingUserPermissions {
            missing_permissions,
            ctx,
            ..
        } => {
            warn!(
                "{} used command {} but misses permissions: {}",
                ctx.author().name,
                ctx.command().name,
                missing_permissions.unwrap()
            );
            ctx.send(CreateReply::default().content("https://tenor.com/view/jurrasic-park-samuel-l-jackson-magic-word-you-didnt-say-the-magic-work-gif-3556977")
            )
            .await
            .unwrap();
        }

        poise::FrameworkError::MissingBotPermissions {
            missing_permissions,
            ctx,
            ..
        } => {
            error!(
                "Bot misses permissions: {} for command {}",
                missing_permissions,
                ctx.command().name
            );

            ctx.send(
                CreateReply::default()
                    .content(format!(
                        "Bot needs the {missing_permissions} permission to perform this command."
                    ))
                    .ephemeral(true),
            )
            .await
            .unwrap();
        }

        poise::FrameworkError::GuildOnly { ctx, .. } => {
            warn!("Guild only command received from outside a guild");
            ctx.say("This does not work outside a guild.")
                .await
                .unwrap();
        }

        poise::FrameworkError::Command { error, ctx: _, .. } => {
            error!("Error in command: {}", error);
        }

        error => {
            error!("Unhandled error on command: {error}");
        }
    }
}
