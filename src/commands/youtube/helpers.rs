use std::{
    sync::{mpsc, Arc},
    time::Duration,
};

use brzthook::{HookListener, Mode};
use poise::serenity_prelude::{self as serenity, ChannelId};
use serde_json::Value;
use time::OffsetDateTime;
use tracing::{debug, error, info, warn};

use super::{queries, SubYtChannel, INVIDIOUS_INSTANCES_URL, YOUTUBE_VIDEO_PREFIX};
use crate::{
    commands::youtube::{YtVideo, EXPIRATION_DAYS},
    db::Db,
    Context, Data, Error,
};

pub(super) async fn get_invidious_instances() -> Result<Option<Vec<Value>>, Error> {
    let response = reqwest::get(INVIDIOUS_INSTANCES_URL).await?.text().await?;
    let instances: serde_json::Value = serde_json::from_str(&response)?;

    // Keep only instance that have available api calls
    let instances = instances.as_array().map(|instances| {
        instances
            .iter()
            .filter(|inst| inst[1]["api"] == true)
            .cloned()
            .collect()
    });

    Ok(instances)
}

pub async fn listen_loop(
    ctx: serenity::Context,
    db: Arc<Db>,
    listener: Arc<HookListener>,
) -> Result<(), Error> {
    // Start TCP listening in another thread and pass it a `Sender<Notification>
    let (tx, rx) = mpsc::channel();
    listener.listen(&tx);

    info!("Starting webhooks listener");
    // Wait for the sender to transfer data
    loop {
        //std::thread::sleep(Duration::from_millis(1000));

        let recv = rx.recv()?;
        match recv {
            Err(e) => error!("Error in HookListener: {e}"),
            Ok(notification) => {
                info!("Got notification from listener");
                let post_channel_ids =
                    queries::get_post_channel_ids(&db, &notification.channel_id).await?;

                for id in post_channel_ids {
                    let message = format!("New video from **{}** !!!", &notification.channel_name);
                    post_video(
                        ctx.clone(),
                        ChannelId(id),
                        &notification.video_id,
                        Some(&message),
                    )
                    .await?;
                }
            }
        }
    }
}

//TODO: check this ugly unwraps
pub async fn expiration_check_timer(listener: Arc<HookListener>, db: Arc<Db>) {
    loop {
        info!("Checking for expiration");
        let subs = queries::get_subs_list(&db).await.unwrap();
        let mut resubbed = vec![];
        let now_utc = OffsetDateTime::now_utc();

        for sub in subs {
            if now_utc > sub.expire_on.checked_add(time::Duration::days(-1)).unwrap()
                && !resubbed.contains(&sub.yt_channel_id)
            {
                listener
                    .subscribe(&sub.yt_channel_id, Mode::Subscribe)
                    .unwrap();
                let new_expire = now_utc
                    .checked_add(time::Duration::days(EXPIRATION_DAYS))
                    .unwrap();
                queries::update_expire_on(&db, new_expire, &sub.yt_channel_id)
                    .await
                    .unwrap();

                info!(
                    "Renewed subscription for {}: new expire_on = {}",
                    &sub.yt_channel_id, new_expire
                );
                resubbed.push(sub.yt_channel_id);
            }
        }

        // Check once a day
        std::thread::sleep(Duration::from_secs(24 * 3600));
    }
}

pub async fn get_name_id(ctx: &Context<'_>, url: &str) -> Result<Option<(String, String)>, Error> {
    let Some(instances) = get_invidious_instances().await? else {
        warn!("No invidious instance found");
        return Ok(None);
    };
    // If the input is the full address https://www.youtube.com/{suffix}
    let suffix = match url.rsplit_once('/') {
        Some(tup) => tup.1,
        None => url,
    };

    // The Youtube channel id starts with "UC", we can call directly the channel endpoint
    // If suffix starts with "@", we use the search endpoint to find the channel with that name
    // (assuming the first result is the good one)
    let query = if suffix.starts_with("UC") {
        format!("/api/v1/channels/{suffix}")
    } else if suffix.starts_with('@') {
        format!("/api/v1/search?q={suffix}&type=channel")
    } else {
        ctx.say("Invalid input").await?;
        return Ok(None);
    };

    let instance_uri = instances[0][1]["uri"].to_string();
    let instance_uri = instance_uri.trim_matches('"');
    let query_url = format!("{instance_uri}{query}");
    let response: Value = reqwest::get(&query_url).await?.json().await?;
    let (author_name, author_id) = if suffix.starts_with('@') {
        (
            response[0]["author"]
                .as_str()
                .ok_or("No author name found")?,
            response[0]["authorId"]
                .as_str()
                .ok_or("No authorId found")?,
        )
    } else {
        (
            response["author"].as_str().ok_or("No author found")?,
            response["authorId"].as_str().ok_or("No authorId found")?,
        )
    };

    Ok(Some((author_name.to_owned(), author_id.to_owned())))
}

//TODO: embed
pub(super) async fn post_video(
    ctx: serenity::Context,
    channel_id: ChannelId, // The discord channel where the video will be posted
    video_id: &str,        // The youtube video id
    message: Option<&str>, // A message that will be write before the video link
) -> Result<(), Error> {
    let url = format!("{YOUTUBE_VIDEO_PREFIX}{video_id}");
    let content = match message {
        Some(m) => format!("{m}\n{url}"),
        None => url,
    };

    channel_id.say(ctx, &content).await?;

    Ok(())
}
