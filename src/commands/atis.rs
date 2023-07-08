use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::lib::{error::Error, utils};

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct AtisResponse {
    airport: String,
    #[serde(rename = "type")]
    _type: String,
    code: String,
    datis: String,
}

async fn fetch_atis(station: &str) -> Result<Vec<AtisResponse>, Error> {
    let url = format!("https://datis.clowd.io/api/{station}");
    let resp = reqwest::get(&url).await?.json().await;

    match resp {
        Ok(data) => {
            let resp: Vec<AtisResponse> = data;
            Ok(resp)
        }
        Err(_) => Err(Error::NotFound("The station code provided does not exist".into())),
    }
}

async fn parse_atis(station: &str) -> String {
    match fetch_atis(station).await {
        Ok(data) => {
            if data.len() == 1 {
                format!(
                    "```
{}
```",
                    data[0].datis
                )
            } else {
                format!(
                    "```
{}

{}
```",
                    data[0].datis, data[1].datis,
                )
            }
        }
        Err(e) => format!("`There was an error retrieving data: {e}`"),
    }
}

#[command]
pub async fn atis(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let args: Vec<String> = args.message().split(' ').map(str::to_uppercase).collect();

    for arg in args {
        match utils::check_station_code(&arg) {
            Ok(_) => {
                let data = parse_atis(&arg).await;
                msg.channel_id.say(&ctx.http, data).await?
            }
            Err(e) => msg.channel_id.say(&ctx.http, format!("`{e}`")).await?,
        };
    }

    Ok(())
}
