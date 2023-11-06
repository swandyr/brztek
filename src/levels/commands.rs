use poise::serenity_prelude as serenity;
use std::time::Instant;
use tracing::{debug, info, instrument};

use super::{
    draw::{self, UserInfoCard},
    queries, DEFAULT_PP_TESSELATION_VIOLET,
};

use crate::Data;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

// Change .webp extension to .png and remove parameters from URL
#[instrument]
fn clean_url(mut url: String) -> String {
    if let Some(index) = url.find("webp") {
        let _: String = url.split_off(index);
        url.push_str("png?size=96"); // Ensure the size of the image to be at max 96x96
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
    let member = user.unwrap_or(ctx.author_member().await.unwrap().into_owned());

    let user_id = member.user.id.0;
    let guild_id = member.guild_id.0;

    // Get user from database
    let db = &ctx.data().db;
    let user_level = queries::get_user(db, user_id, guild_id).await?;

    // Get user info to display on the card
    let username = member
        .display_name()
        .replace(|c: char| !(c.is_alphanumeric() || c.is_whitespace()), "");

    // Request profile picture through HTTP if `avatar_url` is Some().
    // Fallback to a default picture if None.
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
        image::load_from_memory(&bytes)?
    };
    let (image_width, image_height) = (image.width() as usize, image.height() as usize);
    let image_buf = image.into_bytes();

    let user_http = ctx.http().get_user(user_id).await?;
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

    let t_1 = Instant::now();
    ctx.send(|m| {
        let file = serenity::AttachmentType::from((image.as_slice(), "rank_card.png"));
        m.attachment(file)
    })
    .await?;
    info!("Rank card sent in {} µs", t_1.elapsed().as_micros());

    info!("Command processed in {} µs", t_0.elapsed().as_micros());

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

    let number = number.unwrap_or(10);

    // Ensure the message was sent from a guild
    let (guild_id, guild_name) = if let Some(guild) = ctx.guild() {
        (guild.id.0, guild.name)
    } else {
        ctx.say("This does not work outside a guild.").await?;
        return Ok(());
    };

    // Get a vec of all users in database
    let db = &ctx.data().db;
    let mut all_users = queries::get_all_users(db, guild_id).await?;

    // Sort all users by rank
    all_users.sort_by(|a, b| a.rank.cmp(&b.rank));

    let mut top_users = vec![];
    for user in all_users.iter().take(number) {
        let name = ctx
            .http()
            .get_member(guild_id, *user.user_id.as_u64())
            .await?
            .display_name()
            .into_owned()
            .replace(|c: char| !(c.is_alphanumeric() || c.is_whitespace()), "");
        let accent_colour = ctx
            .http()
            .get_user(*user.user_id.as_u64())
            .await?
            .accent_colour
            .unwrap_or(serenity::Colour::LIGHTER_GREY)
            .tuple();
        let user_info_card = UserInfoCard::new(name, user.rank, user.level, user.xp, accent_colour);
        top_users.push(user_info_card);
    }

    // Generate card
    let image = draw::gen_top_card(&top_users, &guild_name).await?;

    // Send generated file
    ctx.send(|b| {
        let file = serenity::AttachmentType::from((image.as_slice(), "top_card.png"));
        b.attachment(file)
    })
    .await?;

    Ok(())
}
