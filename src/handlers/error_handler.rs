use poise::{serenity_prelude as serenity, CreateReply};
use tracing::{error, instrument, trace, warn};

use crate::{misc, Data, Error};

#[instrument(skip(error))]
pub async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    match error {
        poise::FrameworkError::Setup {
            error,
            framework: _,
            data_about_bot,
            ctx: _,
            ..
        } => {
            error!("Error during setup: {error:?}\ndata_about_bot: {data_about_bot:#?}");
        }

        poise::FrameworkError::EventHandler {
            error,
            ctx: _,
            event,
            framework: _,
            ..
        } => {
            error!("Error while handling event {event:?}: {error:?}");
        }

        poise::FrameworkError::UnknownCommand {
            ctx,
            msg,
            msg_content,
            framework,
            ..
        } => {
            // On unknown command, it will first queries the database to check for corresponding
            // entry in the learned table for a user's registered command
            trace!(
                "Unknown command received: {}. Checking for learned commands",
                msg_content
            );
            let db = &framework.user_data.db;
            let guild_id = msg.guild_id.unwrap().get();

            let queried = misc::queries::get_learned(db, msg_content, guild_id)
                .await
                .expect("Query learned_command returned with error");
            if let Some(link) = queried {
                trace!("Learned command found: {}", msg_content);
                msg.channel_id
                    .send_message(&ctx, serenity::CreateMessage::new().content(link))
                    .await
                    .expect("Error sending learned command link");
            } else {
                warn!("Unknown command: {}", msg_content);
                msg.channel_id
                    .send_message(&ctx, serenity::CreateMessage::new().content("https://tenor.com/view/kaamelott-perceval-cest-pas-faux-not-false-gif-17161490"))
                    .await
                    .unwrap();
            }
        }

        poise::FrameworkError::MissingUserPermissions {
            missing_permissions,
            ctx,
            ..
        } => {
            warn!(
                "{} used command {} but misses permissions: {}",
                ctx.author().name,
                ctx.command().name,
                missing_permissions.unwrap()
            );
            ctx.send(CreateReply::default().content("https://tenor.com/view/jurrasic-park-samuel-l-jackson-magic-word-you-didnt-say-the-magic-work-gif-3556977")
            )
            .await
            .unwrap();
        }

        poise::FrameworkError::MissingBotPermissions {
            missing_permissions,
            ctx,
            ..
        } => {
            error!(
                "Bot misses permissions: {} for command {}",
                missing_permissions,
                ctx.command().name
            );

            ctx.send(
                CreateReply::default()
                    .content(format!(
                        "Bot needs the {missing_permissions} permission to perform this command."
                    ))
                    .ephemeral(true),
            )
            .await
            .unwrap();
        }

        poise::FrameworkError::GuildOnly { ctx, .. } => {
            warn!("Guild only command received from outside a guild");
            ctx.say("This does not work outside a guild.")
                .await
                .unwrap();
        }

        poise::FrameworkError::Command { error, ctx, .. } => {
            let content = format!("Error in command: {error}");
            error!(content);
            ctx.send(CreateReply::default().content(content).ephemeral(true))
                .await
                .unwrap();
        }

        error => {
            error!("Unhandled error on command: {error}");
        }
    }
}
