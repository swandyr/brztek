use brzthook::Mode;
use time::Duration;
use tracing::instrument;
use crate::{
    Context, Error,
    youtube::{
        queries,
        func::get_name_id,
        constants::EXPIRATION_DAYS,
        models::SubYtChannel
    }
};

/// Create a new Youtube webhook
///
/// The new videos will be posted in the channel where this command is called from
///
/// name argument takes the address https://www.youtube.com/{id} or https://www.youtube.com/@{name}
#[instrument(skip(ctx))]
#[poise::command(
slash_command,
guild_only,
required_permissions = "MANAGE_WEBHOOKS",
ephemeral,
category = "Youtube"
)]
pub(super) async fn sub(
    ctx: Context<'_>,
    #[description = "Url of the Youtube channel"] url: String,
) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;

    let Some((author_name, author_id)) = get_name_id(&ctx, &url).await? else {
        ctx.say("No channel found").await?;
        return Ok(());
    };

    // Send the subscription request to the hub
    ctx.data()
        .hook_listener
        .subscribe(&author_id, Mode::Subscribe)?;

    let content = format!("Subbed to {author_name}");
    let expire_on = time::OffsetDateTime::now_utc()
        .checked_add(Duration::days(EXPIRATION_DAYS))
        .ok_or("Webhook subscription: cannot set expiration date")?;

    // Store in the database
    let sub = SubYtChannel {
        yt_channel_id: author_id,
        yt_channel_name: author_name,
        guild_id: ctx.guild_id().unwrap().get(),
        post_channel_id: ctx.channel_id().get(),
        expire_on,
    };
    let db = &ctx.data().db;
    queries::insert_sub(db, sub).await?;

    ctx.say(&content).await?;
    Ok(())
}