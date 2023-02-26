use poise::serenity_prelude::{self as serenity, CacheHttp};
use std::time::Instant;
use tracing::{info, instrument};

use crate::levels::cards::{rank_card, top_card};
use crate::Data;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

/// Show your rank
#[instrument]
#[poise::command(prefix_command, slash_command, guild_only, category = "Levels")]
pub async fn rank(ctx: Context<'_>) -> Result<(), Error> {
    let t_0 = Instant::now();

    let user_id = ctx.author().id.0;

    // Ensure the command was sent from a guild channel
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
    let member = ctx.author_member().await.unwrap();
    let username = member.display_name();

    let avatar_url = ctx.author().avatar_url();
    let user_http = ctx.http().get_user(user_id).await?;
    let accent_colour = user_http
        .accent_colour
        .unwrap_or(serenity::Colour::LIGHTER_GREY)
        .tuple();

    // Generate the card that will be save with name "rank.png"
    let t_1 = Instant::now();
    let image = rank_card::gen_user_card(
        &username,
        avatar_url,
        accent_colour,
        user_level.level,
        user_level.rank,
        user_level.xp,
    )
    .await?;
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
#[instrument]
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
            .into_owned();
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
