use poise::{
    serenity_prelude::{self as serenity, Mentionable},
    CreateReply,
};
use rand::{thread_rng, Rng};
use tracing::{debug, info, instrument, warn};

use super::{
    consts::BASE_RFF_PERC,
    func,
    models::{Roulette, ShotKind},
};
use crate::{Context, Error};

/// Put random member in timeout for 60s
///
/// The more you use it, the more you can get caught
#[instrument(skip(ctx))]
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    required_bot_permissions = "MODERATE_MEMBERS",
    category = "Roulette"
)]
pub async fn roulette(ctx: Context<'_>) -> Result<(), Error> {
    let mut author = ctx
        .author_member()
        .await
        .ok_or("No author_member found")?
        .into_owned();
    let author_id = author.user.id;

    let now = serenity::Timestamp::now().unix_timestamp();
    let timeout_timestamp = now + 60;
    let time = serenity::Timestamp::from_unix_timestamp(timeout_timestamp)?;

    let db = &ctx.data().db;

    let rff_check = thread_rng().gen_range(1..=100);
    debug!("Generated RFF Check: {}", rff_check);

    let mut entry = {
        let read = ctx.data().roulette_map.lock().unwrap();
        debug!("{read:#?}");
        read.get(&author_id).copied()
    };
    debug!(
        "Roulette Map entry of user {}: {:?}",
        author.display_name(),
        entry
    );
    let (rff_user_chance, _timestamp) = entry.get_or_insert((BASE_RFF_PERC, now));
    debug!("RFF user chance: {}", rff_user_chance);

    //TODO: reset under a certain period of time ?

    // Author is self timed out if the random rff_check number is below the author's rff_user_chance
    if rff_check <= *rff_user_chance {
        info!("Selfshot check not passed for {}", author.display_name());
        // Returns error if the bot cannot timeout_member, usually because he has administrator status
        let timeout_result = func::timeout_member(ctx, &mut author, time).await;
        // Saves result in db
        let roulette = Roulette {
            timestamp: now,
            caller_id: author_id,
            target_id: author_id,
            rff_triggered: Some(*rff_user_chance),
        };
        debug!("{:#?}", roulette);
        let guild = ctx.guild().as_deref().ok_or("Not in guild")?.clone();
        func::record_roulette(db, &guild, roulette).await?;
        // Generates the image that will be attached to the message
        let image = func::gen_roulette_image(&author, &author, ShotKind::Reverse).await?;

        let file = serenity::CreateAttachment::bytes(image.as_slice(), "kf.png");
        let content = format!("**:man_police_officer: RFF activated at {rff_user_chance}%, you're out. :woman_police_officer:**");
        ctx.send(CreateReply::default().attachment(file).content(content))
            .await?;

        // if timeout_member returned Err, it assumes it is because of administrator priviledges, then notify the member
        if let Err(e) = timeout_result {
            warn!("Timeout member returned: {}", e);
            ctx.say(
                "As you're an administrator, I have no power, but I know you won't abuse the rules",
            )
            .await?;
        }

        // Reset the author's selfshot_perc
        {
            let mut write = ctx.data().roulette_map.lock().unwrap();
            write.entry(author_id).and_modify(|(perc, tstamp)| {
                *perc = BASE_RFF_PERC;
                *tstamp = now;
            });
        }
    } else {
        // Get a random member
        let guild = ctx.guild().as_deref().ok_or("Not in guild")?.clone();
        let members = guild
            .members(ctx, None, None)
            .await?
            .into_iter()
            .filter(|m| !m.user.bot)
            .collect::<Vec<_>>();
        let index = thread_rng().gen_range(0..members.len());
        let mut target = members.get(index).ok_or("No member found")?.clone();
        info!("Randomly selected member: {:?}", target.display_name());

        let timeout_result = func::timeout_member(ctx, &mut target, time).await;
        let roulette = Roulette {
            timestamp: now,
            caller_id: author_id,
            target_id: target.user.id,
            rff_triggered: None,
        };
        func::record_roulette(db, &guild, roulette).await?;

        let is_self_shot = author_id == target.user.id.get();

        let image = func::gen_roulette_image(
            &author,
            &target,
            if is_self_shot {
                ShotKind::SelfShot
            } else {
                ShotKind::Normal
            },
        )
        .await?;

        // Send a message according to a self shot or not
        if is_self_shot {
            ctx.send(
                CreateReply::default()
                    .attachment(serenity::CreateAttachment::bytes(
                        image.as_slice(),
                        "kf.png",
                    ))
                    .content(
                        //"https://tenor.com/view/damn-punch-punching-oops-missed-punch-gif-12199143",
                        "Ouch, looks like it hurts. :sweat_smile:",
                    ),
            )
            .await?;
        } else {
            ctx.send(
                CreateReply::default().attachment(serenity::CreateAttachment::bytes(
                    image.as_slice(),
                    "kf.png",
                )),
            )
            .await?;
        }

        if let Err(e) = timeout_result {
            warn!("Timeout member returned: {}", e);
            ctx.say(format!("The roulette has chosen, {}, but I can't mute you, would you kindly shut up for the next 60 seconds ?", target.mention()))
                .await?;
        }

        // Increase author's rff_user_chance
        {
            let inc = thread_rng().gen_range(2..11);
            let mut write = ctx.data().roulette_map.lock().unwrap();
            write
                .entry(author_id)
                .and_modify(|(rff_perc, tstamp)| {
                    *rff_perc.clamp(&mut 0, &mut 100) += inc;
                    *tstamp = now;
                })
                .or_insert((BASE_RFF_PERC + inc, now));
        }
    }

    Ok(())
}
