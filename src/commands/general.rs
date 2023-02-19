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
    ctx.data().db.set_learned(&name, &link).await?;

    ctx.say(format!("I know {name}")).await?;

    Ok(())
}

#[poise::command(prefix_command, slash_command, category = "General")]
pub async fn learned(ctx: Context<'_>) -> Result<(), Error> {
    let commands = ctx.data().db.get_learned_list().await?;

    let mut content = String::from(">>> List of learned commands: \n");
    for command in commands {
        let line = format!("  - {command}\n");
        content.push_str(&line);
    }

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

#[poise::command(prefix_command, slash_command, category = "General")]
pub async fn bigrig(ctx: Context<'_>) -> Result<(), Error> {
    let url = "https://brfm.radiocloud.pro/api/public/v1/song/current";
    // https://brfm.radiocloud.pro/api/public/v1/song/recent
    let song = reqwest::get(url).await?.json::<Song>().await?;

    info!("Requested bigrig.fm at: {} -> status: {}", url, song.status);

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
