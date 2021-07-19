use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::lib::config;
use crate::lib::error::Error;
use crate::lib::utils;

#[derive(Deserialize, Debug)]
pub struct CurrentResult {
    pub location: Location,
    pub current: Current,
}

#[derive(Deserialize, Debug)]
pub struct Location {
    pub name: String,
    pub region: String,
    pub lat: String,
    pub lon: String,
    pub localtime: String,
}

#[derive(Deserialize, Debug)]
pub struct Current {
    pub temperature: i32,
    pub weather_descriptions: Vec<String>,
    pub wind_speed: i32,
    pub wind_degree: i32,
    pub wind_dir: String,
    pub pressure: i32,
    pub precip: i32,
    pub humidity: i32,
    pub cloudcover: i32,
    pub feelslike: i32,
    pub visibility: i32,
}

pub async fn fetch_current(zip_code: i32) -> Result<CurrentResult, Error> {
    let config = config::Config::load_config()?;
    let url = format!(
        "http://api.weatherstack.com/current?access_key={}&query={}&units=f",
        config.weatherstack, zip_code
    );
    let resp: CurrentResult = reqwest::get(&url).await?.json().await?;

    Ok(resp)
}

async fn parse_current(zip_code: i32) -> String {
    match fetch_current(zip_code).await {
        Ok(data) => {
            let (city, state) = (data.location.name, data.location.region);
            let lat = data.location.lat.parse::<f64>().unwrap();
            let lon = data.location.lon.parse::<f64>().unwrap();
            let pressure = f64::from(data.current.pressure) * 0.0295301;

            format!(
                "```
Current WX => {}, {} (lat: {:.2}, lon: {:.2})

Temperature: {}\u{b0}
Wind Speed: {} MPH
Wind Direction: {} ({}\u{b0})
Pressure: {:.2} Hg
Precipitation: {} IN.
Humidity: {}%
Cloud Cover: {}%
Feels Like: {}\u{b0}
Visbility: {} MI.

Last updated on {}
```",
                city,
                state,
                lat,
                lon,
                data.current.temperature,
                data.current.wind_speed,
                data.current.wind_dir,
                data.current.wind_degree,
                pressure,
                data.current.precip,
                data.current.humidity,
                data.current.cloudcover,
                data.current.feelslike,
                data.current.visibility,
                data.location.localtime
            )
        }
        Err(e) => format!("`There was an error retrieving data: {}`", e),
    }
}

#[command]
pub async fn wx(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let args: Vec<&str> = args.message().split(' ').collect();

    for arg in args {
        match utils::check_zip_code(arg) {
            Ok(zip_code) => {
                let data = parse_current(zip_code).await;
                msg.channel_id.say(&ctx.http, data).await?
            }
            Err(e) => msg.channel_id.say(&ctx.http, format!("`{}`", e)).await?,
        };
    }

    Ok(())
}
