mod helpers;
mod queries;

use brzthook::Mode;
use serde_json::Value;
use time::Duration;
use tracing::{error, info, instrument, warn};

use crate::{Context, Data, Error};
use helpers::get_invidious_instances;
pub use helpers::{expiration_check_timer, get_name_id, listen_loop};

const INVIDIOUS_INSTANCES_URL: &str = "https://api.invidious.io/instances.json?sort_by=health";
const YOUTUBE_VIDEO_PREFIX: &str = "https://www.youtube.com/watch?v=";
const EXPIRATION_DAYS: i64 = 5;

#[derive(Debug, Clone)]
struct SubYtChannel {
    yt_channel_name: String,
    yt_channel_id: String,
    guild_id: u64,
    post_channel_id: u64,
    expire_on: time::OffsetDateTime,
}

#[allow(unused)]
#[derive(Debug, Clone)]
struct YtVideo {
    author_name: String,
    author_id: String,
    video_id: String,
    video_title: String,
}

/// Commands for interacting with Youtube
///
/// Subcommands: `search`, `sub`, `unsub`, `list`
#[poise::command(
    slash_command,
    guild_only,
    subcommands("search", "sub", "unsub", "list", "sub_details"),
    subcommand_required,
    category = "Youtube"
)]
pub async fn yt(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Search a Youtube video.
///
/// The bot will post the first video returned by the search phrase you entered.
///
/// It requests the Invidious API to get the video Id to avoid the need of a Google API Key.
/// The link posted is Youtube though.
#[instrument(skip(ctx))]
#[poise::command(prefix_command, slash_command, category = "Youtube")]
pub async fn search(
    ctx: Context<'_>,
    #[rest]
    #[description = "search input"]
    search: String,
) -> Result<(), Error> {
    ctx.defer().await?;

    // Request available invidious instance
    let Some(instances) = get_invidious_instances().await? else {
        warn!("No invidious instance found");
        ctx.say("No Invidious instance found.").await?;
        return Ok(());
    };

    for instance in instances {
        let Some(instance_uri) = &instance[1]["uri"].as_str() else {
            continue;
        };
        let query_url = format!("{instance_uri}/api/v1/search?q={search}&type=video");

        // Send GET request to the invidious instance
        info!("GET {query_url}");
        let response = reqwest::get(query_url).await?;
        info!("Status: {}", response.status());

        // Process the response if response status code is Ok then exit loop
        // If status code is not Ok, try with next instance
        if response.status().is_success() {
            let json: Value = response.json().await?;
            let Some(video_id) = &json[0]["videoId"].as_str() else {
                continue;
            };
            info!("Found video id: {video_id}");

            let url = format!("{YOUTUBE_VIDEO_PREFIX}{video_id}");
            ctx.say(url).await?;

            return Ok(());
        }
    }

    // If no request to any invidious instance returned with Ok
    ctx.say("Nothing to see here.").await?;
    Ok(())
}

/// Create a new Youtube webhook
///
/// The new videos will be posted in the channel where this command is called from
///
/// name argument takes the address https://www.youtube.com/{id} or https://www.youtube.com/@{name}
#[instrument(skip(ctx))]
#[poise::command(
    slash_command,
    guild_only,
    required_permissions = "MANAGE_WEBHOOKS",
    ephemeral,
    category = "Youtube"
)]
async fn sub(
    ctx: Context<'_>,
    #[description = "Url of the Youtube channel"] url: String,
) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;

    let Some((author_name, author_id)) = get_name_id(&ctx, &url).await? else {
        ctx.say("No channel found").await?;
        return Ok(());
    };

    // Send the subscription request to the hub
    ctx.data()
        .hook_listener
        .subscribe(&author_id, Mode::Subscribe)?;

    let content = format!("Subbed to {author_name}");
    let expire_on = time::OffsetDateTime::now_utc()
        .checked_add(Duration::days(EXPIRATION_DAYS))
        .ok_or("Webhook subscription: cannot set expiration date")?;

    // Store in the database
    let sub = SubYtChannel {
        yt_channel_id: author_id,
        yt_channel_name: author_name,
        guild_id: ctx.guild_id().unwrap().get(),
        post_channel_id: ctx.channel_id().get(),
        expire_on,
    };
    let db = &ctx.data().db;
    queries::insert_sub(db, sub).await?;

    ctx.say(&content).await?;
    Ok(())
}

//* This shit doesn't work */
//TODO: Find a way to make this shit work
// async fn sub_autocomplete<'a>(
//     ctx: Context<'a>,
//     partial: &'a str,
// ) -> impl Stream<Item = String> + 'a {
//     futures::stream::iter(
//         ctx.data()
//             .autocomplete
//             .lock()
//             .unwrap()
//             .clone()
//             .iter()
//             .filter(|&name| name.1 == ctx.guild_id().unwrap().0)
//             .map(|name| name.0.clone()),
//     )
//     .filter(move |name| futures::future::ready(name.starts_with(partial)))
//     .map(|name| name.clone())
// }

/// Unsub and delete a webhook
///
/// Input the exact name of the channel (use "/yt list" if needed)
#[allow(unused)]
#[poise::command(
    slash_command,
    guild_only,
    required_permissions = "MANAGE_WEBHOOKS",
    ephemeral,
    category = "Youtube"
)]
async fn unsub(
    ctx: Context<'_>,
    #[description = "Name of the channel"] name: String,
) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;

    let db = &ctx.data().db;
    let guild_id = ctx.guild_id().ok_or("Not in guild")?;
    let Some(sub) = queries::get_sub(db, &name, guild_id.get()).await? else {
        ctx.say("No channel found with this name.").await?;
        return Ok(());
    };

    let author_id = sub.yt_channel_id;

    ctx.data()
        .hook_listener
        .subscribe(&author_id, Mode::Unsubscribe)?;

    queries::delete_sub(db, &author_id, ctx.guild_id().unwrap().get()).await?;

    let content = format!("Unsubbed to {name}");
    ctx.say(&content).await?;
    Ok(())
}

/// List all subs in the guild
#[poise::command(slash_command, guild_only, category = "Youtube")]
async fn list(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Not in guild")?;
    let subs: Vec<String> = queries::get_subs_list(&ctx.data().db)
        .await?
        .into_iter()
        .filter(|s| s.guild_id == guild_id.get())
        .map(|s| s.yt_channel_name)
        .collect();

    let content = format!("**List of subscribed channels:**\n>>> {}", subs.join("\n"));
    ctx.say(&content).await?;

    Ok(())
}

#[poise::command(slash_command, guild_only, ephemeral, category = "Youtube")]
async fn sub_details(ctx: Context<'_>, id: String) -> Result<(), Error> {
    let callback = ctx.data().config.brzthook.callback.as_str();
    let topic = format!("https://www.youtube.com/xml/feeds/videos.xml?channel_id={id}");
    let content = format!("https://pubsubhubbub.appspot.com/subscription-details?hub.callback={callback}&hub.topic={topic}&hub.secret=");
    ctx.say(&content).await?;
    Ok(())
}
