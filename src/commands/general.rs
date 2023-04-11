use std::collections::HashMap;
use std::string;

use poise::serenity_prelude as serenity;
use poise::serenity_prelude::{CacheHttp, Mentionable, RoleId};
use rand::{prelude::thread_rng, Rng};
use tracing::{debug, info, instrument};

use crate::{draw::roulette_killfeed::gen_killfeed, Data};

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

//TODO: see commands options, for aliases and stuff

/// Ping the bot!
///
/// He'll pong you back.
#[instrument(skip(ctx))]
#[poise::command(prefix_command, slash_command, category = "General")]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("Pong!").await?;
    Ok(())
}

/// Make the bot remember.
///
/// Save a named command with a link the bot will post when responding
/// to the command.
#[instrument(skip(ctx))]
#[poise::command(prefix_command, slash_command, guild_only, category = "General")]
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

/// What the bot learned.
///
/// List all learned command names.
#[instrument(skip(ctx))]
#[poise::command(prefix_command, slash_command, guild_only, category = "General")]
pub async fn learned(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap().0;

    let commands = ctx.data().db.get_learned_list(guild_id).await?;

    let mut content = String::from(">>> List of learned commands: \n");
    let mut content_len = content.len();
    for command in commands {
        let line = format!("  - {command}\n");
        content_len += line.len();

        if content_len <= 2000 {
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
    category = "General"
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
    let role_id = db.get_role_color(guild_id, user_id).await?;

    // Member display name will be the name of the role
    let name = member.display_name();
    let role_name = format!("bot_color_{name}");

    let colour = match hex_colour {
        Some(hex) => {
            if !(hex.len() == 7 && hex.starts_with('#')) {
                ctx.say(format!("Color format should be \"#rrggbb\""))
                    .await?;
                return Ok(());
            }

            if !(hex[1..7].chars().all(|c| c.is_ascii_hexdigit())) {
                ctx.say(format!("{} is not a valid color hex code.", hex))
                    .await?;
                return Ok(());
            }

            let r: u8 = u8::from_str_radix(&hex[1..3], 16)?;
            let g: u8 = u8::from_str_radix(&hex[3..5], 16)?;
            let b: u8 = u8::from_str_radix(&hex[5..7], 16)?;

            serenity::Colour::from_rgb(r, g, b)
        }
        None => {
            // User banner colour will be the colour of the role
            let Some(colour) = ctx.http().get_user(user_id).await?.accent_colour else {
                ctx.say("Cannot find banner color").await?;
                return Ok(());
            };
            colour
        }
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
                    .position(bot_role_position as u8 - 1)
            })
            .await?;
        info!("role_color created: {}", role.id.0);

        // Add the role to the user
        member.to_mut().add_role(ctx, role.id).await?;
        info!("role added to user");

        let role_id = role.id.0;
        db.set_role_color(guild_id, user_id, role_id).await?;
    }

    ctx.send(|b| b.reply(true).content("Done!")).await?;

    Ok(())
}

/// Timeout a member
///
/// Usage: /tempscalme <@User> <duration (default 60)>
/// duration = 0 to disable timeout
#[instrument(skip(ctx))]
#[poise::command(
    slash_command,
    //required_permissions = "MODERATE_MEMBERS",
    required_bot_permissions = "MODERATE_MEMBERS",
    guild_only,
    category = "General"
)]
pub async fn tempscalme(
    ctx: Context<'_>,
    #[description = "User to put in timeout"] mut member: serenity::Member,
    #[description = "Timeout duration (default: 60s)"] duration: Option<i64>,
) -> Result<(), Error> {
    // Cancel timeout
    if let Some(0) = duration {
        member.enable_communication(ctx).await?;

        ctx.say(format!("{} timeout cancelled!", member.mention()))
            .await?;
        info!("timeout cancel");

        return Ok(());
    }

    let now = serenity::Timestamp::now().unix_timestamp();
    let timeout_timestamp = now + duration.unwrap_or(60);
    let time = serenity::Timestamp::from_unix_timestamp(timeout_timestamp)?;

    match member.communication_disabled_until {
        // If to_timestamp > 0, member is already timed out
        Some(to_timestamp) if to_timestamp.unix_timestamp() > now => {
            debug!("to: {} - now: {}", to_timestamp.unix_timestamp(), now);
            info!("already timed out until {}", to_timestamp.naive_local());
            ctx.say(format!(
                "{} is already timed out until {}",
                member.mention(),
                to_timestamp.naive_local()
            ))
            .await?;
        }
        _ => {
            timeout_member(ctx, &mut member, time).await?;

            ctx.say(format!(
                "{} timed out until {}",
                member.mention(),
                time.naive_local(),
            ))
            .await?;
            info!(
                "{} timed out until {}",
                member.display_name(),
                time.naive_local()
            );
        }
    }

    Ok(())
}

/// A random member is in timeout for 60s
#[instrument(skip(ctx))]
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    required_bot_permissions = "MODERATE_MEMBERS",
    category = "General"
)]
pub async fn roulette(ctx: Context<'_>) -> Result<(), Error> {
    // Retrieves the list of members of the guild
    // let channel = ctx.channel_id().to_channel(ctx).await?.guild().unwrap();
    let guild = ctx.guild().unwrap();
    let members = guild
        .members(ctx, None, None)
        .await?
        .into_iter()
        .filter(|m| !m.user.bot)
        .collect::<Vec<_>>();

    let index = thread_rng().gen_range(0..members.len());
    let mut member = members.get(index).unwrap().clone();
    debug!("Randomly selected member: {:?}", member);

    let now = serenity::Timestamp::now().unix_timestamp();
    let timeout_timestamp = now + 60;
    let time = serenity::Timestamp::from_unix_timestamp(timeout_timestamp)?;

    let timeout_result = timeout_member(ctx, &mut member, time).await;

    let user_1 = ctx.author_member().await.unwrap();

    // Store result in db
    let db = &ctx.data().db;
    let guild_id = guild.id.0;
    let user_1_id = user_1.user.id.0;
    let user_2_id = member.user.id.0;

    db.add_roulette_result(guild_id, now, user_1_id, user_2_id)
        .await?;

    // Reply on the channel
    let user_1_name = user_1
        .display_name()
        .replace(|c: char| !(c.is_alphanumeric() || c.is_whitespace()), "");
    let user_2_name = member
        .display_name()
        .replace(|c: char| !(c.is_alphanumeric() || c.is_whitespace()), "");
    let image = gen_killfeed(&user_1_name, &user_2_name)?;

    ctx.send(|m| {
        let file = serenity::AttachmentType::from((image.as_slice(), "kf.png"));
        m.attachment(file)
    })
    .await?;

    if timeout_result.is_err() {
        ctx.say(format!("The roulette has chosen, {}, but I can't mute you, would you kindly shut up for the next 60 seconds ?", member.mention()))
            .await?;
    }

    Ok(())
}

#[instrument(skip(ctx))]
async fn timeout_member(
    ctx: Context<'_>,
    member: &mut serenity::Member,
    time: serenity::Timestamp,
) -> Result<(), Error> {
    member
        .disable_communication_until_datetime(ctx, time)
        .await?;

    Ok(())
}

#[instrument(skip(ctx))]
#[poise::command(slash_command, prefix_command, guild_only, category = "General")]
pub async fn toproulette(ctx: Context<'_>) -> Result<(), Error> {
    let db = &ctx.data().db;
    let guild_id = ctx.guild_id().unwrap().0;
    let scores = db.get_roulette_scores(guild_id).await?;
    info!("Got roulette scores");

    let mut callers_map = HashMap::new();
    let mut targets_map = HashMap::new();

    scores.iter().for_each(|(caller, target)| {
        callers_map
            .entry(caller)
            .and_modify(|x| *x += 1)
            .or_insert(1);
        targets_map
            .entry(target)
            .and_modify(|x| *x += 1)
            .or_insert(1);
    });

    // Process callers
    let mut callers = callers_map
        .iter()
        .map(|(k, v)| (**k, *v))
        .collect::<Vec<(u64, i32)>>();
    callers.sort_by(|a, b| b.1.cmp(&a.1));

    let mut callers_field = String::new();
    for caller in callers.iter().take(10) {
        let member = ctx.http().get_member(guild_id, caller.0).await?;
        let line = format!("{} - {}\n", member.display_name(), caller.1);
        callers_field.push_str(&line);
    }

    // Process targets
    let mut targets = targets_map
        .iter()
        .map(|(k, v)| (**k, *v))
        .collect::<Vec<(u64, i32)>>();
    targets.sort_by(|a, b| b.1.cmp(&a.1));

    let mut target_field = String::new();
    for target in targets.iter().take(10) {
        let member = ctx.http().get_member(guild_id, target.0).await?;
        let line = format!("{} - {}", member.display_name(), target.1);
        target_field.push_str(&line);
    }

    // Send embedded top 10 leaderboard
    ctx.send(|b| {
        b.embed(|f| {
            f.title("Roulette Leaderboard")
                .field("Callers", &callers_field, true)
                .field("Targets", &target_field, true)
        })
    })
    .await?;

    Ok(())
}

const BIGRIG_CURRENT_URL: &str = "https://brfm.radiocloud.pro/api/public/v1/song/current";
//const BIGRIG_RECENT_URL: &str = https://brfm.radiocloud.pro/api/public/v1/song/recent

/// Check if Jolene is playing on BigRig FM
///
/// The bot will show what's now on BigRig, even if it isn't Dolly Parton.
#[allow(dead_code)]
#[instrument(skip(ctx))]
#[poise::command(prefix_command, slash_command, category = "General")]
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
