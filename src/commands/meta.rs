use chrono::Local;
use serenity::framework::standard::{macros::command, CommandError, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::Uptime;

#[command]
pub async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id.say(&ctx.http, "`Pong!`").await?;

    Ok(())
}

#[command]
pub async fn uptime(ctx: &Context, msg: &Message) -> CommandResult {
    let current_time = Local::now();
    let start_time = {
        let data = ctx.data.read().await;

        match data.get::<Uptime>() {
            Some(val) => *val,
            None => return Err(CommandError::from("Unable to load uptime data")),
        }
    };
    let mut formatter = timeago::Formatter::new();

    formatter.num_items(3);
    formatter.ago("");

    let uptime = formatter.convert_chrono(start_time, current_time);

    msg.channel_id.say(&ctx.http, format!("`{}`", uptime)).await?;

    Ok(())
}

#[command]
pub async fn help(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id
        .say(
            &ctx.http,
            "```
Bot Commands

Return current weather          !wx current <zip code>
Return current UV index         !uv current <zip code>
Return forecasted UV index      !uv forecast <zip code>
Return bot uptime               !uptime
This help menu                  !help
```"
            .to_string(),
        )
        .await?;

    Ok(())
}
