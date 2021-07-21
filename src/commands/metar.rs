use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::lib::error::Error;
use crate::lib::utils;

async fn fetch_metar(station: &str) -> Result<String, Error> {
    let url =
        format!("https://tgftp.nws.noaa.gov/data/observations/metar/stations/{}.TXT", station);
    let resp = reqwest::get(&url).await?.text().await?;

    if resp.contains("The requested URL") {
        Err(Error::NotFound("The station code provided does not exist".into()))
    } else {
        Ok(resp)
    }
}

async fn parse_metar(station: &str) -> String {
    match fetch_metar(station).await {
        Ok(data) => {
            let data: Vec<&str> = data.split('\n').filter(|x| x.contains(&station)).collect();
            format!("`{}`", data[0])
        }
        Err(e) => format!("`There was an error retrieving data: {}`", e),
    }
}

#[command]
pub async fn metar(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let args: Vec<String> = args.message().split(' ').map(str::to_uppercase).collect();

    for arg in args {
        match utils::check_station_code(&arg) {
            Ok(_) => {
                let data = parse_metar(&arg).await;
                msg.channel_id.say(&ctx.http, data).await?
            }
            Err(e) => msg.channel_id.say(&ctx.http, format!("`{}`", e)).await?,
        };
    }

    Ok(())
}
