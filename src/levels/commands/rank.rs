use std::time::Instant;
use poise::{CreateReply, serenity_prelude as serenity};
use tracing::{debug, info, instrument};
use crate::{
    Context,
    Error,
    levels::{
        models::UserInfoCard,
        func::{resize_avatar::resize_avatar},
        draw::rank_card,
        queries,
        constants::DEFAULT_PP_TESSELATION_VIOLET}
};

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
        let url = resize_avatar(url);
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
    let image = rank_card::gen_user_card(user_info, (image_width, image_height, &image_buf))?;
    info!("Rank card generated in {} µs", t_1.elapsed().as_micros());

    let t_2 = Instant::now();
    let file = serenity::CreateAttachment::bytes(image.as_slice(), "rank_card.png");
    ctx.send(CreateReply::default().attachment(file)).await?;
    info!("Rank card sent in {} µs", t_2.elapsed().as_micros());

    info!("Command rank processed in {} µs", t_0.elapsed().as_micros());

    Ok(())
}