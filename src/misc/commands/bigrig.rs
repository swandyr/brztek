use poise::{serenity_prelude as serenity, CreateReply};
use tracing::{info, instrument};

use super::consts::BIGRIG_CURRENT_URL;
use crate::{Context, Error};

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

/// Check if Jolene is playing on BigRig FM
///
/// The bot will show what's now on BigRig.
#[allow(dead_code)]
#[instrument(skip(ctx))]
#[poise::command(prefix_command, slash_command, rename = "br", category = "Misc")]
pub async fn bigrig(ctx: Context<'_>) -> Result<(), Error> {
    let song = reqwest::get(BIGRIG_CURRENT_URL)
        .await?
        .json::<Song>()
        .await?;

    info!(
        "Requested bigrig.fm at: {} -> status: {}",
        BIGRIG_CURRENT_URL, song.status
    );

    ctx.send(
        CreateReply::default().embed(
            serenity::CreateEmbed::new()
                .title("Now on BigRig FM")
                .url(&song.data.link)
                .thumbnail(&song.data.album_art)
                .field("Artist", &song.data.artist, false)
                .field("Title", &song.data.title, false)
                .footer(serenity::CreateEmbedFooter::new(&format!(
                    "Play count: {}",
                    song.data.playcount
                ))),
        ),
    )
    .await?;

    Ok(())
}
