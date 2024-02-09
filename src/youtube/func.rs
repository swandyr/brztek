use poise::serenity_prelude::futures::{self, Stream, StreamExt};
use serde_json::Value;
use tracing::warn;

use super::{constants::INVIDIOUS_INSTANCES_URL, queries};
use crate::{Context, Error};

pub async fn autocomplete_sublist<'a>(
    ctx: Context<'_>,
    partial: &'a str,
) -> impl Stream<Item = String> + 'a {
    let db = &ctx.data().db;
    let subs_list = queries::get_subs_list(db).await.unwrap();
    futures::stream::iter(subs_list).map(|sub| sub.yt_channel_name)
}

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

pub async fn get_name_id(ctx: &Context<'_>, url: &str) -> Result<Option<(String, String)>, Error> {
    let Some(instances) = get_invidious_instances().await? else {
        warn!("No invidious instance found");
        return Ok(None);
    };
    // If the input is the full address https://www.youtube.com/{suffix}
    let suffix = url.rsplit_once('/').map_or(url, |tuple| tuple.1);

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
