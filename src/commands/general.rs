use poise::serenity_prelude as serenity;
use tracing::info;

use crate::Data;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[poise::command(prefix_command, slash_command, category = "General")]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("Pong!").await?;
    Ok(())
}

#[poise::command(prefix_command, slash_command, category = "General")]
pub async fn learn(
    ctx: Context<'_>,
    #[description = "Name"] name: String,
    #[description = "Link"] link: String,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap().0;

    ctx.data().db.set_learned(&name, &link, guild_id).await?;

    ctx.say(format!("I know {name}")).await?;

    Ok(())
}

#[poise::command(prefix_command, slash_command, category = "General")]
pub async fn learned(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap().0;

    let commands = ctx.data().db.get_learned_list(guild_id).await?;

    let mut content = String::from(">>> List of learned commands: \n");
    commands.iter().for_each(|command| {
        let line = format!("  - {command}\n");
        content.push_str(&line);
    });

    ctx.say(content).await?;

    Ok(())
}

#[derive(Debug, serde::Deserialize)]
struct SongData {
    id: i32,
    artist: String,
    title: String,
    album_art: String,
    link: String,
    played_at: i32,
    playcount: i32,
}

#[derive(Debug, serde::Deserialize)]
struct Song {
    status: String,
    data: SongData,
}

const BIGRIG_CURRENT_URL: &str = "https://brfm.radiocloud.pro/api/public/v1/song/current";
//const BIGRIG_RECENT_URL: &str = https://brfm.radiocloud.pro/api/public/v1/song/recent

#[poise::command(prefix_command, slash_command, category = "General")]
pub async fn bigrig(ctx: Context<'_>) -> Result<(), Error> {
    let song = reqwest::get(BIGRIG_CURRENT_URL)
        .await?
        .json::<Song>()
        .await?;

    info!(
        "Requested bigrig.fm at: {} -> status: {}",
        BIGRIG_CURRENT_URL, song.status
    );

    ctx.send(|b| {
        b.embed(|f| {
            f.title("Now on BigRig FM")
                .url(&song.data.link)
                .thumbnail(&song.data.album_art)
                .field("Artist", &song.data.artist, false)
                .field("Title", &song.data.title, false)
                .footer(|f| f.text(&format!("Play count: {}", song.data.playcount)))
        })
    })
    .await?;

    Ok(())
}

#[poise::command(prefix_command, slash_command, category = "General")]
pub async fn yt(
    ctx: Context<'_>,
    #[rest]
    #[description = "search input"]
    search: String,
) -> Result<(), Error> {
    // Request available invidious instance
    let instances_url = "https://api.invidious.io/instances.json?sort_by=health";
    let response = reqwest::get(instances_url).await?.text().await?;
    let instances: serde_json::Value = serde_json::from_str(&response)?;

    // Keep only instance that have available api calls
    let instances = instances
        .as_array()
        .unwrap()
        .iter()
        .filter(|inst| inst[1]["api"] == true);

    for instance in instances {
        let instance_uri = &instance[1]["uri"].to_string();
        let instance_uri = instance_uri.trim_matches('"');
        let query_url = format!("{instance_uri}/api/v1/search?q={search}&type=video");

        // Send GET request to the invidious instance
        info!("GET {query_url}");
        let response = reqwest::get(query_url).await?;
        info!("Status: {}", response.status());

        // Process the response if response status code is Ok then exit loop
        // If status code is not Ok, try with next instance
        if response.status() == reqwest::StatusCode::OK {
            let text = response.text().await?;
            let json: serde_json::Value = serde_json::from_str(&text)?;
            let video_id = &json[0]["videoId"].to_string();
            let video_id = video_id.trim_matches('"');
            info!("Found video id: {video_id}");

            let youtube_url = format!("https://www.youtube.com/watch?v={video_id}");
            ctx.say(youtube_url).await?;

            return Ok(());
        }
    }

    // If no request to any invidious instance returned with Ok
    ctx.say("Nothing to see here.").await?;
    Ok(())
}
