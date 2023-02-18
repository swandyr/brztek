use poise::serenity_prelude as serenity;

use crate::Data;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[poise::command(
    prefix_command,
    slash_command,
    //check = "owner_check",
    category = "General"
)]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("Pong!").await?;
    Ok(())
}

#[poise::command(prefix_command, slash_command, category = "General")]
pub async fn learn(
    ctx: Context<'_>,
    #[description = "Name"] name: String,
    #[description = "Link"] link: String,
) -> Result<(), Error> {
    ctx.data().db.set_learned(&name, &link).await?;

    ctx.say(format!("I now know {name}")).await?;

    Ok(())
}

#[poise::command(prefix_command, slash_command, category = "General")]
pub async fn learned(ctx: Context<'_>) -> Result<(), Error> {
    let commands = ctx.data().db.get_learned_list().await?;

    let mut content = String::from(">>> List of learned commands: \n");
    for command in commands {
        let line = format!("  - {command}\n");
        content.push_str(&line);
    }

    ctx.say(content).await?;

    Ok(())
}
