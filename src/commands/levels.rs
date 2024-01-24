mod draw;
pub mod handle_message;
pub mod queries;
pub mod user_level;
pub mod xp_func;

// Xp parameters
const MIN_XP_GAIN: i64 = 15;
const MAX_XP_GAIN: i64 = 25;
const DELAY_ANTI_SPAM: i64 = 60;

// Rank card constants
const CARD_FONT: &str = "Akira Expanded"; // Font needs to be installed on the system (https://www.dafont.com/akira-expanded.font)
const DEFAULT_PP_TESSELATION_VIOLET: &str = "assets/images/default-pp/Tessellation-Violet.png";
const TOP_TITLE_HEIGHT: usize = 60;
const TOP_USER_HEIGHT: usize = 32;

use poise::{serenity_prelude as serenity, CreateReply};
use std::time::Instant;
use tracing::{debug, info, instrument};

use super::to_png_buffer;
use crate::{Context, Data, Error};
use draw::UserInfoCard;

// Change .webp extension to .png and remove parameters from URL
#[instrument]
fn clean_url(mut url: String) -> String {
    if let Some(index) = url.find("webp") {
        let _: String = url.split_off(index);
        url.push_str("png?size=96"); // Ensure the size of the image to be at max 96x96
                                     //url.push_str("png");
    }
    url
}

/// Show your rank
#[instrument(skip(ctx, user), fields(guild=ctx.guild().unwrap().name, author=ctx.author().name))]
#[poise::command(prefix_command, slash_command, guild_only, category = "Levels")]
pub async fn rank(
    ctx: Context<'_>,
    #[description = "The user"] user: Option<serenity::Member>,
) -> Result<(), Error> {
    let t_0 = Instant::now();

    debug!("user: {user:?}");
    let member = user.unwrap_or(
        ctx.author_member()
            .await
            .ok_or("No member found")?
            .into_owned(),
    );

    let user_id = member.user.id.get();
    let guild_id = member.guild_id.get();

    // Get user from database
    let db = &ctx.data().db;
    let user_level = queries::get_user(db, user_id, guild_id).await?;

    // Get user info to display on the card
    let username = member
        .display_name()
        .replace(|c: char| !(c.is_alphanumeric() || c.is_whitespace()), "");

    // Request profile picture through HTTP if `avatar_url` is Some().
    // Fallback to a default picture if None.
    // TODO: use member.face() ?
    let avatar_url = member.user.avatar_url();
    let image = if let Some(url) = avatar_url {
        let url = clean_url(url);
        debug!("avatar url: {url}");
        let bytes = reqwest::get(&url).await?.bytes().await?;
        info!("Received avatar from {url}");
        image::load_from_memory(&bytes)?
    } else {
        let bytes = std::fs::read(DEFAULT_PP_TESSELATION_VIOLET)?;
        info!("Loaded default avatar");
        image::load_from_memory_with_format(&bytes, image::ImageFormat::Png)?
    };
    let (image_width, image_height) = (image.width() as usize, image.height() as usize);
    let image_buf = image.into_bytes();

    let user_http = ctx.http().get_user(user_id.into()).await?;
    let accent_colour = user_http
        .accent_colour
        .unwrap_or(serenity::Colour::LIGHTER_GREY)
        .tuple();

    let user_info = UserInfoCard::new(
        username,
        user_level.rank,
        user_level.level,
        user_level.xp,
        accent_colour,
    );

    // Generate the card
    let t_1 = Instant::now();
    let image = draw::gen_user_card(user_info, (image_width, image_height, &image_buf))?;
    info!("Rank card generated in {} µs", t_1.elapsed().as_micros());

    let t_2 = Instant::now();
    let file = serenity::CreateAttachment::bytes(image.as_slice(), "rank_card.png");
    ctx.send(CreateReply::default().attachment(file)).await?;
    info!("Rank card sent in {} µs", t_2.elapsed().as_micros());

    info!("Command rank processed in {} µs", t_0.elapsed().as_micros());

    Ok(())
}

/// Show the top users of the server
///
/// Default is 10.
#[instrument(skip(ctx), fields(guild=ctx.guild().unwrap().name, author=ctx.author().name))]
#[poise::command(prefix_command, slash_command, guild_only, category = "Levels")]
pub async fn top(
    ctx: Context<'_>,
    #[description = "Number of users (default: 10)"]
    #[min = 1]
    #[max = 30]
    number: Option<usize>,
) -> Result<(), Error> {
    ctx.defer().await?;

    let t_0 = Instant::now();

    let number = number.unwrap_or(10);

    // Ensure the message was sent from a guild
    let guild = ctx.guild().as_deref().cloned().ok_or("Not in guild")?;
    let (guild_id, guild_name) = (guild.id.get(), guild.name.as_str());

    let t_1 = Instant::now();
    // Get a vec of all users in database
    let db = &ctx.data().db;
    let mut all_users = queries::get_all_users(db, guild_id).await?;
    debug!("Got all_users in {} µs", t_1.elapsed().as_micros());

    let t_2 = Instant::now();
    // Sort all users by rank
    all_users.sort_by(|a, b| a.rank.cmp(&b.rank));

    let mut top_users = vec![];
    for user in all_users.iter().take(number) {
        let name = ctx
            .http()
            .get_member(guild_id.into(), user.user_id)
            .await?
            .display_name()
            .replace(|c: char| !(c.is_alphanumeric() || c.is_whitespace()), "");
        let accent_colour = ctx
            .http()
            .get_user(user.user_id)
            .await?
            .accent_colour
            .unwrap_or(serenity::Colour::LIGHTER_GREY)
            .tuple();
        let user_info_card = UserInfoCard::new(name, user.rank, user.level, user.xp, accent_colour);
        top_users.push(user_info_card);
    }
    debug!("Process users infos in {} µs", t_2.elapsed().as_micros());

    let t_3 = Instant::now();
    // Generate card
    let image = draw::gen_top_card(&top_users, guild_name).await?;
    debug!("Generated top card in {} µs", t_3.elapsed().as_micros());

    let t_4 = Instant::now();
    // Send generated file
    let file = serenity::CreateAttachment::bytes(image.as_slice(), "top_card.png");
    ctx.send(CreateReply::default().attachment(file)).await?;
    debug!("Send top card in {} µs", t_4.elapsed().as_micros());

    debug!("Top card processed in {} µs", t_0.elapsed().as_micros());

    Ok(())
}
