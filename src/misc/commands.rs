use poise::serenity_prelude as serenity;
use poise::serenity_prelude::{CacheHttp, RoleId};
use tracing::{info, instrument};

use super::{queries, BIGRIG_CURRENT_URL};
use crate::Data;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

//TODO: see commands options, for aliases and stuff

/// Ping the bot!
///
/// He'll pong you back.
#[instrument(skip(ctx))]
#[poise::command(prefix_command, slash_command, category = "Misc")]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("Pong!").await?;
    Ok(())
}

/// Make the bot remember.
///
/// Save a named command with a link the bot will post when responding
/// to the command.
#[instrument(skip(ctx))]
#[poise::command(prefix_command, slash_command, guild_only, category = "Misc")]
pub async fn learn(
    ctx: Context<'_>,
    #[description = "Name"] name: String,
    #[description = "Link"] link: String,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap().0;
    let db = &ctx.data().db;

    queries::set_learned(db, &name, &link, guild_id).await?;

    ctx.say(format!("I know {name}")).await?;

    Ok(())
}

/// What the bot learned.
///
/// List all learned command names.
#[instrument(skip(ctx))]
#[poise::command(prefix_command, slash_command, guild_only, category = "Misc")]
pub async fn learned(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap().0;
    let db = &ctx.data().db;

    let commands = queries::get_learned_list(db, guild_id).await?;

    let mut content = String::from(">>> List of learned commands: \n");
    let mut content_len = content.len();
    for command in commands {
        let line = format!("- {command}\n");
        content_len += line.len();

        if content_len <= serenity::constants::MESSAGE_CODE_LIMIT {
            // Limit of character accepted in a discord message
            content.push_str(&line);
        } else {
            ctx.say(content).await?;
            content = format!(">>> {line}");
            content_len = content.len();
        }
    }

    ctx.say(content).await?;

    Ok(())
}

/// Get your own role
///
/// Get a personal role with the color of your choice
///
/// Usage: /setcolor <color>
/// where color is in hexadecimal format (eg: #d917d3)
///
/// If no color is given, it will retrieve the profile's banner color
#[instrument(skip(ctx))]
#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    required_bot_permissions = "MANAGE_ROLES",
    ephemeral,
    category = "Misc"
)]
pub async fn setcolor(
    ctx: Context<'_>,
    #[description = "Colour in hexadecimal format"] hex_colour: Option<String>,
) -> Result<(), Error> {
    // Request db for an `Option<u64>` if a role is already attributed to the user
    let db = &ctx.data().db;
    let guild = ctx.guild().unwrap();
    let guild_id = guild.id.0;
    let mut member = ctx.author_member().await.unwrap();
    let user_id = member.user.id.0;
    let role_id = queries::get_role_color(db, guild_id, user_id).await?;

    // Member display name will be the name of the role
    let name = member.display_name();
    let role_name = format!("bot_color_{name}");

    let colour = if let Some(hex) = hex_colour {
        if !(hex.len() == 7 && hex.starts_with('#')) {
            ctx.say("Color format should be \"#rrggbb\"".to_string())
                .await?;
            return Ok(());
        }

        if !(hex[1..7].chars().all(|c| c.is_ascii_hexdigit())) {
            ctx.say(format!("{hex} is not a valid color hex code."))
                .await?;
            return Ok(());
        }

        let r: u8 = u8::from_str_radix(&hex[1..3], 16)?;
        let g: u8 = u8::from_str_radix(&hex[3..5], 16)?;
        let b: u8 = u8::from_str_radix(&hex[5..7], 16)?;

        serenity::Colour::from_rgb(r, g, b)
    } else {
        // User banner colour will be the colour of the role
        let Some(colour) = ctx.http().get_user(user_id).await?.accent_colour else {
                ctx.say("Cannot find banner color").await?;
                return Ok(());
            };
        colour
    };

    info!("role_id: {:?}", role_id);
    if let Some(id) = role_id {
        guild
            .edit_role(ctx, RoleId(id), |r| {
                r.colour(colour.0 as u64).name(role_name)
            })
            .await?;
        info!("role_color {} updated", id);
    } else {
        let bot_role_position = guild.role_by_name("brztek").unwrap().position;
        info!("bot role position: {}", bot_role_position);
        let role = guild
            .create_role(ctx, |role| {
                role.name(role_name)
                    .colour(colour.0 as u64)
                    .permissions(serenity::Permissions::empty())
                    .position(u8::try_from(bot_role_position).unwrap() - 1)
            })
            .await?;
        info!("role_color created: {}", role.id.0);

        // Add the role to the user
        member.to_mut().add_role(ctx, role.id).await?;
        info!("role added to user");

        let role_id = role.id.0;
        queries::set_role_color(db, guild_id, user_id, role_id).await?;
    }

    ctx.send(|b| b.reply(true).content("Done!")).await?;

    Ok(())
}

/// Check if Jolene is playing on BigRig FM
///
/// The bot will show what's now on BigRig, even if it isn't Dolly Parton.
#[allow(dead_code)]
#[instrument(skip(ctx))]
#[poise::command(prefix_command, slash_command, category = "Misc", rename = "br")]
pub async fn bigrig(ctx: Context<'_>) -> Result<(), Error> {
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

/// Search a Youtube video.
///
/// The bot will post the first video returned by the search phrase you entered.
///
/// It requests the Invidious API to get the video Id to avoid the need of a Google API Key.
/// The link posted is Youtube though.
#[instrument(skip(ctx))]
#[poise::command(prefix_command, slash_command, category = "Misc")]
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
