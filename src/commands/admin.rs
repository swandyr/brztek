use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::{channel::Message, prelude::PartialMember, user::User, Permissions},
    prelude::*,
};
use tracing::{error, info};

use crate::utils::db::Db;

// Check if the author of the message has admin permissions
async fn is_admin(ctx: &Context, member: &PartialMember) -> bool {
    member.roles.iter().any(|r| {
        r.to_role_cached(&ctx.cache)
            .map_or(false, |r| r.has_permission(Permissions::ADMINISTRATOR))
    })
}

#[command]
#[description = "Check if you have administrator permissions"]
pub async fn am_i_admin(ctx: &Context, msg: &Message) -> CommandResult {
    let is_admin = if let Some(member) = &msg.member {
        is_admin(ctx, member).await
    } else {
        false
    };

    let content = if is_admin {
        String::from("Yes, you are!")
    } else {
        String::from("No, you're not.")
    };

    msg.reply(&ctx.http, content).await?;

    Ok(())
}

// -------------- Admin Xp Commands -------------------
// Retrieve a user from username in the guild
// Possibility to set level and/or xp

#[command]
#[description = "Set a user's Xp"]
pub async fn setxp(ctx: &Context, msg: &Message, arg: Args) -> CommandResult {
    todo!()
}

#[command]
#[description = "Set a user's level"]
pub async fn setlevel(ctx: &Context, msg: &Message, arg: Args) -> CommandResult {
    todo!()
}

async fn get_user(ctx: &Context) -> Option<User> {
    todo!()
}

#[command]
#[description = "Clear database"]
pub async fn delete_ranks(ctx: &Context, msg: &Message) -> CommandResult {
    if let Some(member) = &msg.member {
        if !is_admin(ctx, member).await {
            msg.channel_id
                .send_message(&ctx.http, |m| {
                    m.content("You don't have the permission to do that")
                })
                .await?;
            return Ok(());
        }
    }

    let data = ctx.data.read().await;
    let db = data.get::<Db>().expect("Expected Db in TypeMap.");

    if let Some(guild_id) = msg.guild_id {
        info!("Delete rows in table 'edn_ranks'");
        db.delete_table(guild_id.0).await?;

        msg.channel_id.say(&ctx.http, "All xp dropped to 0").await?;
    } else {
        error!("No guild_id found");
    }

    Ok(())
}

// ------------ Configuration Parameters --------------
use tracing::debug;

//use crate::utils::config::Config;

#[derive(Debug, Clone, Copy)]
enum Parameters {
    SpamDelay,
    MinXpGain,
    MaxXpGain,
}
use Parameters::*;

impl TryFrom<&str> for Parameters {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "spam_delay" => Ok(SpamDelay),
            "min_xp_gain" => Ok(MinXpGain),
            "max_xp_gain" => Ok(MaxXpGain),
            _ => Err("Parameters::try_from() returned with error: invalid value"),
        }
    }
}

impl std::fmt::Display for Parameters {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SpamDelay => write!(f, "spam delay"),
            MinXpGain => write!(f, "min xp gain"),
            MaxXpGain => write!(f, "max xp gain"),
        }
    }
}

#[command]
#[description = "Get or set configuration parameters"]
pub async fn config(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if let Some(arg) = args.current() {
        let parameter = Parameters::try_from(arg)?;
        args.advance();
        let value = args.current();

        if handle_command(ctx, msg, parameter, value).await.is_err() {
            msg.channel_id
                .send_message(&ctx.http, |m| m.content("Argument is invalid."))
                .await?;
        }
    } else {
        let data = ctx.data.read().await;
        let db = data.get::<Db>().expect("Expected Db in TypeMap");
        let guild_id = msg.guild_id.unwrap().0;

        let (spam_delay, min_xp_gain, max_xp_gain) = db.get_config(guild_id).await?;
        let content = format!(
            r#"> **Server config:**
        > spam_delay: {spam_delay}
        > min_xp_gain {min_xp_gain}
        > max_xp_gain {max_xp_gain}"#
        );

        msg.channel_id
            .send_message(&ctx.http, |m| m.content(&content))
            .await?;
    }

    Ok(())
}

async fn handle_command(
    ctx: &Context,
    msg: &Message,
    parameter: Parameters,
    value: Option<&str>,
) -> CommandResult {
    let guild_id = msg.guild_id.unwrap().0;

    if let Some(val) = value {
        set_parameter(ctx, parameter, val, guild_id).await?;

        let content = format!(
            "Changing {} to {}",
            parameter,
            get_parameter(ctx, parameter, guild_id).await?
        );
        msg.channel_id
            .send_message(&ctx.http, |m| m.content(content))
            .await?;
    } else {
        {
            let content = format!(
                "{} is set to {}.",
                parameter,
                get_parameter(ctx, parameter, guild_id).await?
            );
            msg.channel_id
                .send_message(&ctx.http, |m| m.content(content))
                .await?;
        }
    }

    Ok(())
}

async fn set_parameter(
    ctx: &Context,
    parameter: Parameters,
    value: &str,
    guild_id: u64,
) -> anyhow::Result<()> {
    // Acquire a write lock on the data
    let data = ctx.data.write().await;
    debug!("Data lock acquired.");
    let db = data.get::<Db>().expect("Expected Db in TypeMap");

    match parameter {
        SpamDelay => db.set_spam_delay(guild_id, value.parse()?).await?,
        MinXpGain => db.set_min_xp_gain(guild_id, value.parse()?).await?,
        MaxXpGain => db.set_max_xp_gain(guild_id, value.parse()?).await?,
    };

    // // Get mut ref of the config
    // let config = data.get::<Config>().expect("Expected Config in TypeMap.");
    // debug!("Get mut Config.");

    // // Acquire a write lock on the config
    // let mut lock = config.write().await;
    // debug!("Config lock acquired.");

    // // Internal of Config are mutable
    // match parameter {
    //     SpamDelay => {
    //         lock.xp_settings.delay_anti_spam = value.parse::<i64>()?;
    //         debug!("delay set to {value}.");
    //     }
    //     MinXpGain => {
    //         lock.xp_settings.min_xp_gain = value.parse::<i64>()?;
    //         debug!("min xp gain set to {value}.");
    //     }
    //     MaxXpGain => {
    //         lock.xp_settings.max_xp_gain = value.parse::<i64>()?;
    //         debug!("max xp gain set to {value}.");
    //     }
    // }

    // Drop acquired locks
    // drop(lock);
    // debug!("Config lock droped.");
    // drop(data);
    // debug!("Data lock droped.");

    Ok(())
}

async fn get_parameter(
    ctx: &Context,
    parameter: Parameters,
    guild_id: u64,
) -> Result<String, anyhow::Error> {
    let data = ctx.data.read().await;
    let db = data.get::<Db>().expect("Expected Db in TypeMap");

    let value = match parameter {
        SpamDelay => db.get_spam_delay(guild_id).await?,
        MinXpGain => db.get_min_xp_gain(guild_id).await?,
        MaxXpGain => db.get_max_xp_gain(guild_id).await?,
    };

    // let config = data.get::<Config>().unwrap();
    // let lock = config.read().await;

    // let value = match parameter {
    //     SpamDelay => lock.xp_settings.delay_anti_spam.to_string(),
    //     MinXpGain => lock.xp_settings.min_xp_gain.to_string(),
    //     MaxXpGain => lock.xp_settings.max_xp_gain.to_string(),
    // };

    Ok(value.to_string())
}
