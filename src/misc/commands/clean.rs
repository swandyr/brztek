use poise::CreateReply;
use tracing::{info, instrument};
use crate::{Context, Error};
use crate::clearurl::clear_url;

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