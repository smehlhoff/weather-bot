use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::lib::error::Error;

pub async fn fetch_metar(station: &str) -> Result<String, Error> {
    let url =
        format!("https://tgftp.nws.noaa.gov/data/observations/metar/stations/{}.TXT", station);
    let resp = reqwest::get(&url).await?.text().await?;

    if resp.contains("The requested URL") {
        Err(Error::NotFound("The METAR provided does not exist".into()))
    } else {
        Ok(resp)
    }
}

fn check_metar(station: &str) -> Result<(), Error> {
    if station.len() == 4 {
        if station.starts_with('K') {
            Ok(())
        } else {
            Err(Error::Invalid("U.S. METARs only (e.g., KSFO, KJFK)".into()))
        }
    } else {
        Err(Error::Invalid("The METAR provided is invalid".into()))
    }
}

async fn parse_metar(station: &str) -> String {
    match fetch_metar(station).await {
        Ok(data) => {
            let data: Vec<&str> = data.split('\n').filter(|x| x.contains(&station)).collect();
            format!("`{}`", data[0].to_string())
        }
        Err(e) => format!("`{}`", e),
    }
}

#[command]
pub async fn metar(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let args: Vec<String> = args.message().split(' ').map(str::to_uppercase).collect();

    for arg in args {
        match check_metar(&arg) {
            Ok(_) => {
                let data = parse_metar(&arg).await;
                msg.channel_id.say(&ctx.http, data).await?
            }
            Err(e) => {
                msg.channel_id
                    .say(&ctx.http, format!("`There was an error retrieving data: {}`", e))
                    .await?
            }
        };
    }

    Ok(())
}
