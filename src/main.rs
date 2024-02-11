#![allow(
    clippy::unused_async,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_lossless,
    unused
)]

mod admin;
mod builtins;
mod clearurl;
mod config;
mod database;
mod handlers;
mod levels;
mod mention_roles;
mod misc;
mod roulette;
mod util;
mod youtube;

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
use database::Db;

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
                .with_ansi(false)
                .with_line_number(true)
                .with_writer(non_blocking),
        );
    tracing::subscriber::set_global_default(subscriber)?;

    let token = env::var("DISCORD_TOKEN")?;

    let intents = serenity::GatewayIntents::GUILDS
        | serenity::GatewayIntents::GUILD_MEMBERS
        | serenity::GatewayIntents::GUILD_PRESENCES
        | serenity::GatewayIntents::GUILD_MESSAGES
        | serenity::GatewayIntents::DIRECT_MESSAGES
        | serenity::GatewayIntents::MESSAGE_CONTENT;

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
            admin::commands::import_mee6_levels(),
            admin::commands::set_xp(),
            admin::commands::shutdown(),
            levels::commands::rank(),
            levels::commands::top(),
            mention_roles::commands::gimmeroles(),
            mention_roles::commands::mention_roles(),
            misc::commands::bigrig(),
            misc::commands::clean(),
            misc::commands::learn(),
            misc::commands::learned(),
            misc::commands::ping(),
            misc::commands::setcolor(),
            roulette::commands::rffstar(),
            roulette::commands::roulette(),
            roulette::commands::statroulette(),
            roulette::commands::toproulette(),
            youtube::commands::yt(),
        ],
        event_handler: |ctx, event, framework, user_data| {
            Box::pin(handlers::on_event(ctx, event, framework, user_data))
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
                database::increment_cmd(&ctx.data().db, &ctx.command().qualified_name, guild_id)
                    .await
                    .unwrap();
                info!(
                    "{} executed command {} in guild {:?}",
                    ctx.author().name,
                    ctx.command().qualified_name,
                    guild_name
                );
            })
        },
        on_error: |error| Box::pin(handlers::on_error(error)),
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
