mod admin;
mod builtins;
mod db;
mod handlers;
mod levels;
mod misc;
mod roulette;

use poise::serenity_prelude::{self as serenity, UserId};
use std::{
    collections::HashMap,
    env,
    sync::{Arc, RwLock},
    time::Instant,
};
use tracing::{debug, error, info, instrument, warn};
use tracing_subscriber::{fmt, layer::SubscriberExt, EnvFilter};

use db::Db;

type Error = Box<dyn std::error::Error + Send + Sync>;
// type Context<'a> = poise::Context<'a, Data, Error>;

const PREFIX: &str = "$";

/// Store shared data
#[derive(Debug)]
pub struct Data {
    pub db: Arc<Db>,
    // Hashmap<UserId, (selfshot_perc, timestamp)
    pub roulette_map: Arc<RwLock<HashMap<UserId, (u8, i64)>>>,
}

// ----------------------------------------- Main -----------------------------------------

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
                .with_writer(non_blocking),
        );
    tracing::subscriber::set_global_default(subscriber)?;

    let token = env::var("DISCORD_TOKEN")?;

    let intents = serenity::GatewayIntents::non_privileged()
        | serenity::GatewayIntents::MESSAGE_CONTENT
        | serenity::GatewayIntents::GUILD_MEMBERS
        | serenity::GatewayIntents::GUILD_PRESENCES;

    let db_url = env::var("DATABASE_URL")?;
    let db = Db::new(&db_url).await;
    db.run_migrations().await?;

    let options = poise::FrameworkOptions {
        commands: vec![
            admin::commands::admin(),
            admin::commands::import_mee6_levels(),
            builtins::help(),
            builtins::register(),
            levels::commands::rank(),
            levels::commands::top(),
            misc::commands::bigrig(),
            misc::commands::learn(),
            misc::commands::learned(),
            misc::commands::ping(),
            misc::commands::setcolor(),
            misc::commands::yt(),
            roulette::commands::rffstar(),
            roulette::commands::roulette(),
            roulette::commands::statroulette(),
            roulette::commands::toproulette(),
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
    poise::Framework::builder()
        .token(token)
        .intents(intents)
        .options(options)
        .setup(|_ctx, _data_about, _framework| {
            Box::pin(async move {
                Ok(Data {
                    db: Arc::new(db),
                    roulette_map: Arc::new(RwLock::new(HashMap::new())),
                })
            })
        })
        .run()
        .await?;

    Ok(())
}

// ------------------------------------- Event handler -----------------------------------------

#[instrument(skip_all, fields(event_type=event.name()))]
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

        poise::Event::CacheReady { guilds } => {
            let db = &user_data.db;

            for guild in guilds {
                let guild_id = guild.0;
                db::add_guild(db, guild_id).await?;
                let permissions = guild
                    .member(ctx, framework.bot_id)
                    .await?
                    .permissions(ctx)?;
                info!("Guild: {:?} - id: {}", guild.name(ctx), guild);
                info!("Permissions: {:#?}", permissions);
            }
        }

        poise::Event::Message { new_message } => {
            let t_0 = Instant::now();

            // Do not handle message from bot users
            if new_message.author.bot {
                return Ok(());
            }

            // Ensure the command was sent from a guild channel
            if new_message.guild_id.is_none() {
                return Ok(());
            };

            handlers::message_handler(new_message, ctx, user_data).await?;

            debug!("Message processed in: {} µs", t_0.elapsed().as_micros());
        }

        //? Discord already do this
        poise::Event::GuildMemberAddition { new_member } => {
            handlers::member_addition_handler(new_member, ctx).await?;
        }

        poise::Event::GuildMemberRemoval {
            guild_id,
            user,
            member_data_if_available: _,
        } => {
            handlers::member_removal_handler(guild_id, user, ctx).await?;
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
            // On unknown command, it will firt queries the database to check for correspondant
            // entry in the learned table for a user's registered command
            let db = &framework.user_data.db;
            let guild_id = msg.guild_id.unwrap().0;

            let queried = misc::queries::get_learned(db, msg_content, guild_id)
                .await
                .expect("Query learned_command returned with error");
            if let Some(link) = queried {
                msg.channel_id
                    .send_message(&ctx, |m| m.content(link))
                    .await
                    .expect("Error sending learned command link");
            } else {
                msg.channel_id
                    .send_message(&ctx, |m| m.content("https://tenor.com/view/kaamelott-perceval-cest-pas-faux-not-false-gif-17161490"))
                    .await
                    .unwrap();
            }
        }

        poise::FrameworkError::MissingUserPermissions {
            missing_permissions,
            ctx,
        } => {
            warn!(
                "{} used command {} but misses permissions: {}",
                ctx.author().name,
                ctx.command().name,
                missing_permissions.unwrap()
            );
            ctx.send(|f| {
                f.content("https://tenor.com/view/jurrasic-park-samuel-l-jackson-magic-word-you-didnt-say-the-magic-work-gif-3556977")
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

            ctx.send(|f| {
                f.content(format!(
                    "Bot needs the {missing_permissions} permission to perform this command."
                ))
                .ephemeral(true)
            })
            .await
            .unwrap();
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
            warn!("Unhandled error on command: {error}");
        }
    }
}
