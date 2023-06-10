mod admin;
mod builtins;
mod db;
mod levels;
mod misc;
mod roulette;

use clearurl::clear_url;
use poise::serenity_prelude::{
    self as serenity,
    audit_log::{Action, MemberAction},
    Mentionable, UserId,
};
use rand::{prelude::thread_rng, Rng};
use std::{
    collections::HashMap,
    env,
    sync::{Arc, RwLock},
    time::Instant,
};
use tracing::{debug, error, info, instrument, warn};
use tracing_subscriber::EnvFilter;

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
    pub rff_star: Arc<RwLock<Option<(UserId, u8)>>>,
}

// ----------------------------------------- Main -----------------------------------------

#[instrument]
#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenvy::dotenv()?;
    let filter = EnvFilter::from_default_env();
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .pretty()
        .init();

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
            roulette::commands::topvictims(),
            roulette::commands::topbullies(),
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
                    rff_star: Arc::new(RwLock::new(None)),
                })
            })
        })
        .run()
        .await?;

    Ok(())
}

// ------------------------------------- Event handler -----------------------------------------

#[instrument(skip(ctx, framework, user_data))]
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
                db.create_config_entry(guild_id).await?;
                let permissions = guild
                    .member(ctx, framework.bot_id)
                    .await?
                    .permissions(ctx)?;
                debug!("Permissions: \n{:#?}", permissions);
            }
        }

        poise::Event::Message { new_message } => {
            let t_0 = Instant::now();

            // Do not handle message from bot users
            if new_message.author.bot {
                return Ok(());
            }

            // Ensure the command was sent from a guild channel
            let Some(guild_id) = new_message.guild_id else {
                return Ok(());
            };

            let user_id = new_message.author.id;
            let channel_id = new_message.channel_id;

            // Split the message content on whitespace and new line char
            let content = new_message.content.split(&[' ', '\n']);
            // Filter on any links contained in the message content
            let links = content
                .filter(|f| f.starts_with("https://") || f.starts_with("http://"))
                .collect::<Vec<&str>>();
            for link in links {
                if let Some(cleaned) = clear_url(link).await? {
                    // Send message with cleaned url
                    let content = format!("Cleaned that shit for you\n{cleaned}");
                    channel_id.say(ctx, content).await?;

                    // Delete embeds in original message
                    channel_id
                        .message(ctx, new_message.id)
                        .await?
                        // ctx cache return NotAuthor error, but ctx.http works fine
                        .suppress_embeds(&ctx.http)
                        .await?;
                }
            }

            // User gains xp on message
            levels::handle_message::add_xp(ctx, user_data, &guild_id, &channel_id, &user_id)
                .await?;

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

            let system_channel_id = new_member
                .guild_id
                .to_guild_cached(ctx)
                .unwrap()
                .system_channel_id
                .unwrap();
            system_channel_id
                .send_message(&ctx.http, |m| m.content(content))
                .await?;
        }

        poise::Event::GuildMemberRemoval {
            guild_id,
            user,
            member_data_if_available: _,
        } => {
            let username = format!("{}{}", user.name, user.discriminator);
            //let mut content = format!("RIP **{username}**, you'll be missed");
            let mut content = format!("✝️ RIP en paix **{username}** , un 👼 parti trop tôt 🕯️");

            let system_channel_id = guild_id
                .to_guild_cached(ctx)
                .unwrap()
                .system_channel_id
                .unwrap();

            // if bot can read audit logs
            if guild_id
                .to_guild_cached(ctx)
                .unwrap()
                .role_by_name("brztek")
                .unwrap()
                .has_permission(serenity::Permissions::VIEW_AUDIT_LOG)
            {
                let audit_logs = guild_id
                    .audit_logs(&ctx.http, None, None, None, Some(1))
                    .await
                    .unwrap();
                let last_log = audit_logs.entries.first().unwrap();

                // if last action is the kick of the user, change message content accordingly
                if matches!(last_log.action, Action::Member(MemberAction::Kick)) {
                    if let Some(target_id) = last_log.target_id {
                        if target_id == user.id.0 {
                            content = format!("**{username}** has got his ass out of here!");
                        }
                    }
                }
            }

            system_channel_id
                .send_message(&ctx.http, |m| m.content(content))
                .await?;
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
