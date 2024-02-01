mod list;
mod search;
mod sub;
mod sub_details;
mod unsub;

use super::{constants, func, queries};
use crate::{Context, Error};

use list::list;
use search::search;
use sub::sub;
use sub_details::sub_details;
use unsub::unsub;

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
