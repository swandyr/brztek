pub mod create;
pub mod delete;
pub mod gimmeroles;

use tracing::instrument;

use super::queries;
use crate::{Context, Error};

pub use create::create;
pub use delete::delete;
pub use gimmeroles::gimmeroles;

#[instrument(skip(ctx))]
#[poise::command(
    slash_command,
    required_permissions = "MANAGE_ROLES",
    category = "Mention Roles",
    subcommands("create", "delete")
)]
pub async fn mention_roles(ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}
