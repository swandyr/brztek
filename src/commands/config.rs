use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
    prelude::*,
};
use tracing::debug;

use crate::utils::config::Config;

#[derive(Debug, Clone, Copy)]
enum Parameters {
    SpamDelay,
    MinXpGain,
    MaxXpGain,
    TestString,
}
use Parameters::*;

impl TryFrom<String> for Parameters {
    type Error = &'static str;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "spam_delay" => Ok(SpamDelay),
            "min_xp_gain" => Ok(MinXpGain),
            "max_xp_gain" => Ok(MaxXpGain),
            "test_string" => Ok(TestString),
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
            TestString => write!(f, "test_string"),
        }
    }
}

#[command]
pub async fn config(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let parameter = Parameters::try_from(args.single::<String>()?)?;
    let value = args.current();

    if handle_command(ctx, msg, parameter, value).await.is_err() {
        msg.channel_id
            .send_message(&ctx.http, |m| m.content("Argument is invalid."))
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
    if let Some(val) = value {
        set_parameter(ctx, parameter, val).await?;

        let content = format!(
            "Changing {} to {}",
            parameter,
            &get_parameter(ctx, parameter).await?
        );
        msg.channel_id
            .send_message(&ctx.http, |m| m.content(content))
            .await?;
    } else {
        {
            let content = format!(
                "{} is set to {}.",
                parameter,
                &get_parameter(ctx, parameter).await?
            );
            msg.channel_id
                .send_message(&ctx.http, |m| m.content(content))
                .await?;
        }
    }

    Ok(())
}

async fn set_parameter(ctx: &Context, parameter: Parameters, value: &str) -> anyhow::Result<()> {
    // Acquire a write lock on the data
    let mut data = ctx.data.write().await;
    debug!("Data lock acquired.");

    // Get mut ref of the config
    let config = data
        .get_mut::<Config>()
        .expect("Expected Config in TypeMap.");
    debug!("Get mut Config.");

    // Acquire a write lock on the config
    let mut lock = config.write().await;
    debug!("Config lock acquired.");

    // Internal of Config are mutable
    match parameter {
        SpamDelay => {
            lock.xp_settings.delay_anti_spam = value.parse::<i64>()?;
            debug!("delay set to {value}.");
        }
        MinXpGain => {
            lock.xp_settings.min_xp_gain = value.parse::<i64>()?;
            debug!("min xp gain set to {value}.");
        }
        MaxXpGain => {
            lock.xp_settings.max_xp_gain = value.parse::<i64>()?;
            debug!("max xp gain set to {value}.");
        }
        TestString => {
            lock.test_string = value.to_string();
            debug!("test string set to {value}");
        }
    }

    // Drop acquired locks
    // drop(lock);
    // debug!("Config lock droped.");
    // drop(data);
    // debug!("Data lock droped.");

    Ok(())
}

async fn get_parameter(ctx: &Context, parameter: Parameters) -> Result<String, anyhow::Error> {
    let data = ctx.data.read().await;
    let config = data.get::<Config>().unwrap();
    let lock = config.read().await;

    let value = match parameter {
        SpamDelay => lock.xp_settings.delay_anti_spam.to_string(),
        MinXpGain => lock.xp_settings.min_xp_gain.to_string(),
        MaxXpGain => lock.xp_settings.max_xp_gain.to_string(),
        TestString => lock.test_string.to_string(),
    };

    Ok(value)
}
