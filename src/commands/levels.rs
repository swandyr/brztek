use poise::serenity_prelude::{self as serenity, CacheHttp};
use std::time::Instant;
use tracing::{debug, info};

use crate::levels::cards::{rank_card, top_card, DEFAULT_PP_TESSELATION_VIOLET};
use crate::Data;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

// Change .webp extension to .png and remove parameters from URL
fn clean_url(mut url: String) -> String {
    if let Some(index) = url.find("webp") {
        let _ = url.split_off(index);
        url.push_str("png?size=96"); // Ensure the size of the image to be at max 96x96
    }
    url
}

/// Show your rank
#[poise::command(prefix_command, slash_command, guild_only, category = "Levels")]
pub async fn rank(
    ctx: Context<'_>,
    #[description = "The user"] user: Option<serenity::Member>,
) -> Result<(), Error> {
    let t_0 = Instant::now();

    debug!("user: {user:?}");
    let member = user.unwrap_or(ctx.author_member().await.unwrap().into_owned());

    let user_id = member.user.id.0; // Ensure the command was sent from a guild channel
    let guild_id = if let Some(id) = ctx.guild_id() {
        id.0
    } else {
        ctx.say("This does not work outside a guild.").await?;
        return Ok(());
    };

    // Get user from database
    let user_level = ctx.data().db.get_user(user_id, guild_id).await?;

    // Get user info to display on the card
    //let username = format!("{}#{}", ctx.author().name, ctx.author().discriminator);
    let username = member
        .display_name()
        .replace(|c: char| !c.is_alphanumeric(), "");

    // Request profile picture through HTTP if `avatar_url` is Some().
    // Fallback to a default picture if None.
    let avatar_url = member.user.avatar_url();
    let image = if let Some(url) = avatar_url {
        let url = clean_url(url);
        debug!("avatar url: {url}");
        let bytes = reqwest::get(&url).await?.bytes().await?;
        info!("Received avater from {url}");
        image::load_from_memory(&bytes)?
    } else {
        let default_file = DEFAULT_PP_TESSELATION_VIOLET;
        let bytes = std::fs::read(default_file)?;
        info!("Loaded defaut avatar");
        image::load_from_memory(&bytes)?
    };
    let (image_width, image_height) = (image.width() as usize, image.height() as usize);
    let image_buf = image.into_bytes();

    let user_http = ctx.http().get_user(user_id).await?;
    let accent_colour = user_http
        .accent_colour
        .unwrap_or(serenity::Colour::LIGHTER_GREY)
        .tuple();

    // Generate the card that will be save with name "rank.png"
    let t_1 = Instant::now();
    let image = rank_card::gen_user_card(
        &username,
        (image_width, image_height, &image_buf),
        accent_colour,
        user_level.level,
        user_level.rank,
        user_level.xp,
    )?;
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
#[poise::command(prefix_command, slash_command, guild_only, category = "Levels")]
pub async fn top(
    ctx: Context<'_>,
    #[description = "Number of users (default: 10)"]
    #[min = 1]
    #[max = 30]
    number: Option<usize>,
) -> Result<(), Error> {
    let number = number.unwrap_or(10);

    // Ensure the message was sent from a guild
    let (guild_id, guild_name) = if let Some(guild) = ctx.guild() {
        (guild.id.0, guild.name)
    } else {
        ctx.say("This does not work outside a guild.").await?;
        return Ok(());
    };

    // Get a vec of all users in database
    let mut all_users = ctx.data().db.get_all_users(guild_id).await?;

    // Sort all users by rank
    all_users.sort_by(|a, b| a.rank.cmp(&b.rank));

    let mut top_users = vec![];
    for user in all_users.iter().take(number) {
        let name = ctx
            .http()
            .get_member(guild_id, user.user_id)
            .await?
            .display_name()
            .into_owned()
            .replace(|c: char| !c.is_alphanumeric(), "");
        let user_tup = (name, user.rank, user.level, user.xp);
        top_users.push(user_tup);
    }

    // Generate an image that is saved with name "top_ten.png"
    let image = top_card::gen_top_card(&top_users, &guild_name).await?;

    // Send generated "top_ten.png" file
    ctx.send(|b| {
        let file = serenity::AttachmentType::from((image.as_slice(), "top_card.png"));
        b.attachment(file)
    })
    .await?;

    Ok(())
}
