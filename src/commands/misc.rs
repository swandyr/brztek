pub mod queries;

const BIGRIG_CURRENT_URL: &str = "https://brfm.radiocloud.pro/api/public/v1/song/current";
//const BIGRIG_RECENT_URL: &str = https://brfm.radiocloud.pro/api/public/v1/song/recent

use poise::{serenity_prelude as serenity, CreateReply};
use tracing::{info, instrument};

use crate::{clearurl::clear_url, Data};
use crate::{Context, Error};

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
    let guild_id = ctx.guild_id().ok_or("Not in guild")?.get();
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
    let guild_id = ctx.guild_id().ok_or("Not in guild")?.get();
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
    let guild = ctx.guild().as_deref().cloned().ok_or("Not in guild")?;
    let guild_id = guild.id.get();
    let mut member = ctx.author_member().await.ok_or("author_member not found")?;
    let user_id = member.user.id.get();
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
        let Some(colour) = ctx.http().get_user(user_id.into()).await?.accent_colour else {
            ctx.say("Cannot find banner color").await?;
            return Ok(());
        };
        colour
    };

    info!("role_id: {:?}", role_id);
    if let Some(id) = role_id {
        guild
            .edit_role(
                ctx,
                id,
                serenity::EditRole::new()
                    .colour(colour.0 as u64)
                    .name(role_name),
            )
            .await?;
        info!("role_color {} updated", id);
    } else {
        let bot_role_position = guild.role_by_name("brztek").unwrap().position;
        info!("bot role position: {}", bot_role_position);
        let role = guild
            .create_role(
                ctx,
                serenity::EditRole::new()
                    .name(role_name)
                    .colour(colour.0 as u64)
                    .permissions(serenity::Permissions::empty())
                    .position(bot_role_position - 1),
            )
            .await?;
        info!("role_color created: {}", role.id.get());

        // Add the role to the user
        member.to_mut().add_role(ctx, role.id).await?;
        info!("role added to user");

        let role_id = role.id.get();
        queries::set_role_color(db, guild_id, user_id, role_id).await?;
    }

    ctx.send(CreateReply::default().reply(true).content("Done!"))
        .await?;

    Ok(())
}

/// Explicitly call clean_url
#[instrument(skip(ctx))]
#[poise::command(slash_command, category = "Misc")]
pub async fn clean(
    ctx: Context<'_>,
    #[rest]
    #[description = "Link"]
    links: String,
) -> Result<(), Error> {
    let links = links
        .split(&[' ', '\n'])
        .filter(|f| f.starts_with("https://") || f.starts_with("http://"))
        .collect::<Vec<&str>>();

    if links.is_empty() {
        ctx.send(
            CreateReply::default()
                .content("No valid link provided")
                .ephemeral(true),
        )
        .await?;
    } else {
        for link in links {
            info!("Cleaning link: {link}");
            if let Some(cleaned) = clear_url(link).await? {
                info!("Cleaned link -> {cleaned}");
                // Send message with cleaned url
                ctx.say(cleaned).await?;
            }
        }
    }

    Ok(())
}

/// Check if Jolene is playing on BigRig FM
///
/// The bot will show what's now on BigRig.
#[allow(dead_code)]
#[instrument(skip(ctx))]
#[poise::command(prefix_command, slash_command, category = "Misc")]
pub async fn br(ctx: Context<'_>) -> Result<(), Error> {
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
