use poise::{serenity_prelude as serenity, CreateReply};
use std::time::Instant;
use tracing::{debug, instrument};

use super::{draw::top_card, models::UserInfoCard, queries};
use crate::{Context, Error};

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
    let image = top_card::gen_top_card(&top_users, guild_name).await?;
    debug!("Generated top card in {} µs", t_3.elapsed().as_micros());

    let t_4 = Instant::now();
    // Send generated file
    let file = serenity::CreateAttachment::bytes(image.as_slice(), "top_card.png");
    ctx.send(CreateReply::default().attachment(file)).await?;
    debug!("Send top card in {} µs", t_4.elapsed().as_micros());

    debug!("Top card processed in {} µs", t_0.elapsed().as_micros());

    Ok(())
}
