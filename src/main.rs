#[allow(unused_imports)]
use log::{debug, error, info};
use serenity::{
    async_trait,
    framework::standard::{macros::group, StandardFramework},
    model::{
        channel::Message,
        gateway::Ready,
        prelude::{Member, Mention},
    },
    prelude::*,
};
use std::{env, sync::Arc};

mod utils;
use utils::db::Db;

mod commands;
use commands::{
    general::{HELLO_COMMAND, PING_COMMAND, SAY_COMMAND},
    help::HELP,
    ranking::{DELETE_RANKS_COMMAND, RANK_COMMAND, TOP_COMMAND},
};

#[group]
#[commands(ping, hello, say)]
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
        let db = ctx.data.read().await.get::<Db>().unwrap().clone();
        let mut user = db.get_user(user_id).await.unwrap();
        println!("FOUND: {user:?}");

        user.gain_xp();
        db.update_user(&user).await.unwrap();
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
    }

    #[allow(dead_code, unused_variables)]
    async fn guild_member_addition(&self, ctx: Context, new_member: Member) {}
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
        println!("An error occured while running the client: {why}");
    }
}
