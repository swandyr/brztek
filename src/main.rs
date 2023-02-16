use rand::{prelude::thread_rng, Rng};
use serenity::{
    async_trait,
    framework::standard::{macros::group, StandardFramework},
    http::Http,
    model::{
        application::interaction::{Interaction, InteractionResponseType},
        channel::Message,
        gateway::Ready,
        prelude::{GuildId, Member, Mention, User},
    },
    prelude::*,
};
use std::{collections::HashSet, env, sync::Arc, time::Instant};
use tracing::{debug, error, info};

mod utils;
use utils::{config::Config, db::Db};

mod commands;
mod hooks;
mod levels;
mod slash_commands;

use commands::{
    admin::{AM_I_ADMIN_COMMAND, CONFIG_COMMAND, DELETE_RANKS_COMMAND},
    general::{LEARN_COMMAND, PING_COMMAND},
    help::HELP,
    levels::{RANK_COMMAND, TOP_COMMAND},
};

#[group]
#[summary = "General commands"]
#[commands(ping, learn)]
struct General;

#[group]
#[only_in(guilds)]
#[summary = "Levels & rank commands"]
#[description = "Show your personal rank or the top 10 most active users in the server"]
#[commands(rank, top)]
struct Levels;

#[group]
#[only_in(guilds)]
#[summary = "Admin commands"]
#[commands(config, am_i_admin, delete_ranks)]
struct Administrators;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("{} is connected.", ready.user.name);

        let guild_id_int = 1068933063935004722;
        let guild_id = GuildId(guild_id_int);

        // Register slash commands
        let _commands = GuildId::set_application_commands(&guild_id, &ctx.http, |commands| {
            commands
                .create_application_command(|command| {
                    slash_commands::general::learn::register(command)
                })
                .create_application_command(|command| slash_commands::admin::set::register(command))
        })
        .await;
    }

    async fn cache_ready(&self, ctx: Context, guilds: Vec<GuildId>) {
        let data = ctx.data.read().await;
        let db = data.get::<Db>().expect("Expected Db in TypeMap");

        for guild in guilds {
            if let Err(why) = db.create_config_entry(guild.0).await {
                error!("create_config_entry returned error for guild {guild:#?} : {why}");
            }
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            debug!("Received command interaction: {:#?}", command);

            // Respond to slash commands
            let content = match command.data.name.as_str() {
                "learn" => slash_commands::general::learn::run(&ctx, &command.data.options).await,
                "set" => {
                    slash_commands::admin::set::run(
                        &ctx,
                        &command.data.options,
                        &command.guild_id.unwrap(),
                    )
                    .await
                }
                _ => "Not implemented :(".to_string(),
            };

            if let Err(why) = command
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| message.content(content))
                })
                .await
            {
                error!("Cannot responde to slash command: {why}");
            }
        }
    }

    async fn message(&self, ctx: Context, msg: Message) {
        let t_0 = Instant::now();

        // Prevent the bot to reply to itself
        // if msg.is_own(&ctx.cache) {
        //     return;
        // }

        // Prevent handling bot's message
        // if msg.author.bot {
        //     return;
        // }

        // Ensure the command was sent from a guild channel
        let guild_id = if let Some(id) = msg.guild_id {
            id
        } else {
            return;
        };
        let user_id = msg.author.id;
        let channel_id = msg.channel_id;

        if let Err(why) = levels::handle_message_xp(&ctx, &guild_id, &channel_id, &user_id).await {
            error!("handle_message_xp returned error: {why}");
        }

        info!("Message processed in : {} µs", t_0.elapsed().as_micros());
    }

    async fn guild_member_addition(&self, ctx: Context, new_member: Member) {
        use serenity::constants::JOIN_MESSAGES;

        let index = thread_rng().gen_range(0..JOIN_MESSAGES.len());
        let mention = new_member.mention();
        let content = JOIN_MESSAGES
            .get(index)
            .unwrap()
            .replace("$user", &format!("{mention}"));

        // TODO: Store channel id in database
        if let Ok(chan) = env::var("GENERAL_CHANNEL_ID") {
            if let Ok(id) = chan.parse::<u64>() {
                ctx.cache
                    .guild_channel(id)
                    .unwrap()
                    .send_message(&ctx, |m| m.content(content))
                    .await
                    .unwrap();
            } else {
                error!("Unable to parse GENERAL_CHANNEL_ID; check var in .env file.");
            }
        } else {
            error!("Unable to find GENERAL_CHANNEL_ID; check var in .env file.");
        };
    }

    async fn guild_member_removal(
        &self,
        ctx: Context,
        _guild_id: GuildId,
        user: User,
        _member_data_if_available: Option<Member>,
    ) {
        let username = format!("{}{}", user.name, user.discriminator);
        let content = format!("RIP **{username}**, you'll be missed.");
        if let Ok(chan) = env::var("GENERAL_CHANNEL_ID") {
            if let Ok(id) = chan.parse::<u64>() {
                ctx.cache
                    .guild_channel(id)
                    .unwrap()
                    .send_message(&ctx.http, |m| m.content(content))
                    .await
                    .unwrap();
            } else {
                error!("Unable to parse GENERAL_CHANNEL_ID; check var in .env file.");
            }
        } else {
            error!("Unable to find GENERAL_CHANNEL_ID; check var in .env file.");
        }
    }
}

// ----------------------------------------- Main -----------------------------------------

#[tokio::main]
async fn main() {
    dotenvy::dotenv().expect("Failed to load .env file");
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_target(false)
        .init();

    let token = env::var("DISCORD_TOKEN").expect("token needed");
    let intents = GatewayIntents::non_privileged()
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILDS
        | GatewayIntents::GUILD_MEMBERS;

    let http = Http::new(&token);
    let (owners, bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            if let Some(team) = info.team {
                owners.insert(team.owner_user_id);
            } else {
                owners.insert(info.owner.id);
            }
            match http.get_current_user().await {
                Ok(bot_id) => (owners, bot_id.id),
                Err(why) => {
                    error!("Could not access the bot id: {why}");
                    panic!()
                }
            }
        }
        Err(why) => {
            error!("Could not access application info: {why}");
            panic!()
        }
    };

    let framework = StandardFramework::new()
        .configure(|c| c.prefix("!").on_mention(Some(bot_id)).owners(owners))
        .group(&GENERAL_GROUP)
        .group(&LEVELS_GROUP)
        .group(&ADMINISTRATORS_GROUP)
        .help(&HELP)
        .after(hooks::after)
        .unrecognised_command(hooks::unknown_command);

    let db_url = env::var("DATABASE_URL").expect("database path not found");
    let db = Db::new(&db_url).await;
    db.run_migrations().await.expect("Unable to run migrations");
    // Set config entry if not exists

    // let mut config = Config::load().unwrap_or_else(|err| {
    //     error!("Can't read config file: {err}");
    //     Config::default()
    // });

    let handler = Handler;

    let mut client = Client::builder(token, intents)
        .event_handler(handler)
        .framework(framework)
        .await
        .expect("Error creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<Db>(Arc::new(db));
        // data.insert::<Config>(Arc::new(RwLock::new(config)));
    }

    if let Err(why) = client.start().await {
        error!("An error occured while running the client: {why}");
    }
}
