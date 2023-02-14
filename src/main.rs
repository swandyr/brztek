use rand::{prelude::thread_rng, Rng};
use serenity::{
    async_trait,
    framework::standard::{macros::group, StandardFramework},
    http::Http,
    model::{
        application::{
            command::Command,
            interaction::{Interaction, InteractionResponseType},
        },
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
mod slash_commands;
use commands::{
    admin::{AM_I_ADMIN_COMMAND, CONFIG_COMMAND, DELETE_RANKS_COMMAND},
    general::{LEARN_COMMAND, PING_COMMAND},
    help::HELP,
    hooks::{after, unknown_command},
    ranking::{RANK_COMMAND, TOP_COMMAND},
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
            id.0
        } else {
            return;
        };

        let user_id = msg.author.id.0;
        // let channel_id = msg.channel_id.0;

        let data = ctx.data.read().await;
        // https://github.com/launchbadge/sqlx/issues/2252#issuecomment-1364244820
        let db = data.get::<Db>().expect("Expected Db in TypeMap");

        match db.get_user(user_id, guild_id).await {
            Ok(mut user) => {
                let config = data.get::<Config>().unwrap();
                let xp_settings = config.read().await.xp_settings;

                let has_gained_xp = user.gain_xp_if_not_spam(xp_settings);

                if user.has_level_up() {
                    if let Err(why) = msg
                        .channel_id
                        .send_message(&ctx.http, |m| {
                            let mention = Mention::from(msg.author.id);
                            let message = format!("Level Up, {mention}!");
                            m.content(&message)
                        })
                        .await
                    {
                        error!("Error on send message: {why}");
                    }
                }
                if has_gained_xp {
                    if let Err(why) = db.update_user(&user, guild_id).await {
                        error!("Cannot update user {user_id}:{why}");
                    }
                }

                debug!("User : {user:#?}");
            }
            Err(why) => {
                error!("Cannot get user {user_id} from database: {why}");
            }
        }

        if let Err(e) = update_users_ranks(&ctx, guild_id).await {
            error!("Error in update_all_users_levels: {e}");
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

async fn update_users_ranks(ctx: &Context, guild_id: u64) -> anyhow::Result<()> {
    let t_0 = Instant::now();

    let data = ctx.data.read().await;
    let db = data.get::<Db>().expect("Expected Db in TypeMap");

    // Get a Vec of all users in database
    let mut all_users = db.get_all_users(guild_id).await?;

    // Sort user by descendant xp
    all_users.sort_by(|a, b| b.xp.cmp(&a.xp));

    let mut rank_has_changed = vec![];
    for (i, user) in &mut all_users.iter_mut().enumerate() {
        if user.rank != i as i64 + 1 {
            user.rank = i as i64 + 1;
            rank_has_changed.push(*user)
        }
    }

    if !rank_has_changed.is_empty() {
        db.update_ranks(&rank_has_changed, guild_id).await?;
    }

    info!("Updated all ranks in : {} µs", t_0.elapsed().as_micros());

    Ok(())
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
        .after(after)
        .unrecognised_command(unknown_command);

    let db_url = env::var("DATABASE_URL").expect("database path not found");
    let db = Db::new(&db_url).await;
    db.run_migrations().await.expect("Unable to run migrations");

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
