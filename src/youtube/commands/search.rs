use serde_json::Value;
use tracing::{info, instrument, warn};

use super::{constants::YOUTUBE_VIDEO_PREFIX, func::get_invidious_instances};
use crate::{Context, Error};

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
