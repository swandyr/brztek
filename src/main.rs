use rand::Rng;
use serenity::{
    async_trait,
    framework::standard::{macros::group, StandardFramework},
    model::{
        channel::Message,
        gateway::Ready,
        prelude::{GuildId, Member, Mention, User},
    },
    prelude::*,
};
use std::{env, sync::Arc};
#[allow(unused_imports)]
use tracing::{debug, error, info};

mod utils;
use utils::db::Db;

mod commands;
use commands::{
    general::{HELLO_COMMAND, PING_COMMAND, WELCOME_COMMAND},
    help::HELP,
    ranking::{DELETE_RANKS_COMMAND, RANK_COMMAND, TOP_COMMAND},
};

#[group]
#[commands(ping, hello, welcome)]
pub struct General;

#[group]
#[description = "Command relatable to xp and levels"]
#[summary = "Leveling stuff"]
#[commands(rank, top, delete_ranks)]
pub struct Ranking;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        info!("{} is connected.", ready.user.name);
    }

    async fn message(&self, ctx: Context, msg: Message) {
        // Prevent the bot to reply to itself
        // if msg.is_own(&ctx.cache) {
        //     return;
        // }

        // Prevent handling bot's message
        // if msg.author.bot {
        //     return;
        // }

        let user_id = msg.author.id.0;
        // let channel_id = msg.channel_id.0;

        // https://github.com/launchbadge/sqlx/issues/2252#issuecomment-1364244820
        if let Some(db) = ctx.data.read().await.get::<Db>() {
            let get_user = db.get_user(user_id).await;
            match get_user {
                Ok(mut user) => {
                    let has_gained_xp = user.gain_xp();
                    if user.level_up() {
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
                        if let Err(why) = db.update_user(&user).await {
                            error!("Cannot update user {user_id}:{why}");
                        }
                    }
                }
                Err(why) => {
                    error!("Cannot get user {user_id} from database: {why}");
                }
            }
        }
    }

    async fn guild_member_addition(&self, ctx: Context, new_member: Member) {
        use rand::prelude::thread_rng;
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

#[tokio::main]
async fn main() {
    dotenv::dotenv().expect("Failed to load .env file");
    tracing_subscriber::fmt::init();

    let framework = StandardFramework::new()
        .configure(|c| c.prefix("!"))
        .group(&GENERAL_GROUP)
        .group(&RANKING_GROUP)
        .help(&HELP);

    let token = env::var("DISCORD_TOKEN").expect("token needed");
    let intents = GatewayIntents::non_privileged()
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILDS
        | GatewayIntents::GUILD_MEMBERS;

    let db_url = env::var("DATABASE_URL").expect("database path not found");
    let db = Db::new(&db_url).await;
    db.run_migrations().await.expect("Unable to run migrations");

    let handler = Handler;

    let mut client = Client::builder(token, intents)
        .event_handler(handler)
        .framework(framework)
        .await
        .expect("Error creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<Db>(Arc::new(db));
    }

    if let Err(why) = client.start().await {
        error!("An error occured while running the client: {why}");
    }
}
