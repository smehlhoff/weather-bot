use chrono::prelude::*;
use csv::WriterBuilder;
use serenity::{
    framework::standard::{macros::command, CommandError, CommandResult},
    model::prelude::*,
    prelude::*,
};
use tokio::fs::File;

use crate::{lib::db, BotAdmin, Database, Uptime};

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
            None => return Err(CommandError::from("Error retrieving uptime data")),
        }
    };
    let mut formatter = timeago::Formatter::new();

    formatter.num_items(3);
    formatter.ago("");

    let uptime = formatter.convert_chrono(start_time, current_time);

    msg.channel_id.say(&ctx.http, format!("`{uptime}`")).await?;

    Ok(())
}

#[command]
pub async fn logs(ctx: &Context, msg: &Message) -> CommandResult {
    let admin = {
        let data = ctx.data.read().await;

        match data.get::<BotAdmin>() {
            Some(val) => *val,
            None => return Err(CommandError::from("Error retrieving bot admin data")),
        }
    };

    if msg.author.id.0 == admin {
        let pool = {
            let data = ctx.data.read().await;

            match data.get::<Database>().cloned() {
                Some(val) => val,
                None => return Err(CommandError::from("Error retrieving database pool")),
            }
        };

        match db::fetch_log(&pool).await {
            Ok(logs) => {
                let timestamp: DateTime<Utc> = Utc::now();
                let file_name =
                    format!("./attachments/{}_logs.csv", timestamp.format("%y_%m_%d_%H%M%S"));
                let mut writer = WriterBuilder::new().from_path(&file_name)?;

                for log in logs {
                    writer.serialize(log)?;
                }

                writer.flush()?;

                let file = match File::open(file_name).await {
                    Ok(f) => f,
                    Err(e) => {
                        msg.channel_id
                            .say(&ctx.http, format!("`Error reading csv file: {e}`"))
                            .await?;
                        return Ok(());
                    }
                };
                let file = vec![(&file, "logs.csv")];

                msg.channel_id.send_files(&ctx.http, file, |m| m.content("")).await?
            }
            Err(e) => {
                msg.channel_id
                    .say(&ctx.http, format!("`There was an error retrieving data: {e}`"))
                    .await?
            }
        };

        Ok(())
    } else {
        msg.channel_id.say(&ctx.http, "`You must be the bot admin to run command`").await?;

        Ok(())
    }
}

#[command]
pub async fn help(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id
        .say(
            &ctx.http,
            "```
Bot Commands

Return current weather                  !wx current <zip code>
Return weather forecast                 !wx forecast <zip code>
Return temp forecast in graph format    !wx graph <zip code>
Return METAR report                     !metar <station code>
Return TAF report                       !taf <station code>
Return ATIS information                 !atis <station code>
Return current UV index                 !uv current <zip code>
Return UV index forecast                !uv forecast <zip code>
Return current weather alerts           !alerts <zip code>
Set default location                    !location set <zip code>
Return default location                 !location list
Delete default location                 !location delete
Return bot uptime                       !uptime
Return bot logs (admin only)            !logs
This help menu                          !help
```"
            .to_string(),
        )
        .await?;

    Ok(())
}
