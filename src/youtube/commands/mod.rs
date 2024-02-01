mod search;
mod sub;
mod unsub;
mod list;
mod sub_details;

use crate::{Context, Error};
use search::search;
use sub::sub;
use unsub::unsub;
use list::list;
use sub_details::sub_details;

/// Commands for interacting with Youtube
///
/// Subcommands: `search`, `sub`, `unsub`, `list`
#[poise::command(
slash_command,
guild_only,
subcommands("search", "sub", "unsub", "list", "sub_details"),
subcommand_required,
category = "Youtube"
)]
pub async fn yt(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}