use chrono::prelude::*;
use csv::WriterBuilder;
use serenity::framework::standard::{macros::command, CommandError, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;
use std::fs;
use tokio::fs::File;

use crate::lib::db;
use crate::{AdminBot, Database, Uptime};

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

        match data.get::<AdminBot>() {
            Some(val) => *val,
            None => return Err(CommandError::from("Error retrieving admin bot data")),
        }
    };

    if msg.author.id.0 == admin {
        let pool = {
            let data = ctx.data.read().await;
            data.get::<Database>().expect("Error retrieving database pool").clone()
        };

        match db::fetch_log(&pool).await {
            Ok(logs) => {
                fs::create_dir_all("./attachments")
                    .expect("Error creating ./attachments directory");

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
                        msg.channel_id.say(&ctx.http, format!("`{e}`")).await?;
                        return Ok(());
                    }
                };
                let file = vec![(&file, "logs.csv")];

                msg.channel_id.send_files(&ctx.http, file, |m| m.content("")).await?
            }
            Err(e) => msg.channel_id.say(&ctx.http, format!("`{e}`")).await?,
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
Return current weather alerts           !alerts <zone code>
Return bot uptime                       !uptime
Return bot logs (admin only)            !logs
This help menu                          !help
```"
            .to_string(),
        )
        .await?;

    Ok(())
}
