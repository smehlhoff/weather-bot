use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::lib::error::Error;
use crate::lib::utils;

async fn fetch_taf(station: &str) -> Result<String, Error> {
    let url = format!("https://tgftp.nws.noaa.gov/data/forecasts/taf/stations/{}.TXT", station);
    let resp = reqwest::get(&url).await?.text().await?;

    if resp.contains("The requested URL") {
        Err(Error::NotFound("The station code provided does not exist".into()))
    } else {
        Ok(resp)
    }
}

async fn parse_taf(station: &str) -> String {
    match fetch_taf(station).await {
        Ok(data) => {
            let mut data: Vec<&str> =
                data.split('\n').filter(|x| !x.is_empty()).map(str::trim).collect();
            // Remove datetime element
            data.remove(0);
            format!("```{}```", data.join("\n\t"))
        }
        Err(e) => format!("`There was an error retrieving data: {}`", e),
    }
}

#[command]
pub async fn taf(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let args: Vec<String> = args.message().split(' ').map(str::to_uppercase).collect();

    for arg in args {
        match utils::check_station_code(&arg) {
            Ok(_) => {
                let data = parse_taf(&arg).await;
                msg.channel_id.say(&ctx.http, data).await?
            }
            Err(e) => msg.channel_id.say(&ctx.http, format!("`{}`", e)).await?,
        };
    }

    Ok(())
}
