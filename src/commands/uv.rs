use chrono::prelude::*;
use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::commands::wx;
use crate::lib::config;
use crate::lib::error;

#[derive(Deserialize, Debug)]
struct CurrentResult {
    result: UVCurrent,
}

#[derive(Deserialize, Debug)]
struct UVCurrent {
    uv: f64,
    uv_time: chrono::DateTime<Utc>,
    uv_max: f64,
    uv_max_time: chrono::DateTime<Utc>,
    safe_exposure_time: SafeExposureTime,
    sun_info: SunInfo,
}

#[derive(Deserialize, Debug)]
struct SafeExposureTime {
    st1: Option<i32>,
    st2: Option<i32>,
    st3: Option<i32>,
}

#[derive(Deserialize, Debug)]
struct SunInfo {
    sun_times: SunTimes,
}

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
struct SunTimes {
    sunrise: chrono::DateTime<Utc>,
    solarNoon: chrono::DateTime<Utc>,
    sunset: chrono::DateTime<Utc>,
}

#[derive(Deserialize, Debug)]
struct ForecastResult {
    result: Vec<ForecastPeriod>,
}

#[derive(Deserialize, Debug)]
struct ForecastPeriod {
    uv: f64,
    uv_time: chrono::DateTime<Utc>,
}

async fn fetch_location(zip_code: i32) -> (String, String, f64, f64) {
    let data = wx::fetch_current(zip_code).await.unwrap();
    let city = data.location.name;
    let state = data.location.region;
    let lat = data.location.lat.parse::<f64>().unwrap();
    let lon = data.location.lon.parse::<f64>().unwrap();

    (city, state, lat, lon)
}

async fn fetch_current(lat: f64, lon: f64) -> Result<CurrentResult, error::Error> {
    let config = config::Config::load_config()?;
    let url = format!("https://api.openuv.io/api/v1/uv?lat={}&lng={}", lat, lon);
    let client = reqwest::Client::new();
    let resp: CurrentResult =
        client.get(&url).header("x-access-token", config.openuv).send().await?.json().await?;

    Ok(resp)
}
async fn fetch_forecast(lat: f64, lon: f64) -> Result<ForecastResult, error::Error> {
    let config = config::Config::load_config()?;
    let url = format!("https://api.openuv.io/api/v1/forecast?lat={}&lng={}", lat, lon);
    let client = reqwest::Client::new();
    let resp: ForecastResult =
        client.get(&url).header("x-access-token", config.openuv).send().await?.json().await?;

    Ok(resp)
}

pub async fn parse_current(zip_code: i32) -> String {
    let (city, state, lat, lon) = fetch_location(zip_code).await;

    match fetch_current(lat, lon).await {
        Ok(data) => {
            #[allow(unused_assignments)]
            let mut v = Vec::new();
            let (uv_time, uv_max_time, sunrise, sun_noon, sun_set) = {
                let v2 = vec![
                    data.result.uv_time,
                    data.result.uv_max_time,
                    data.result.sun_info.sun_times.sunrise,
                    data.result.sun_info.sun_times.solarNoon,
                    data.result.sun_info.sun_times.sunset,
                ];

                v = v2
                    .iter()
                    .map(|x| Local.from_utc_datetime(&x.naive_local()).format("%I:%M %p"))
                    .collect();

                (&v[0], &v[1], &v[2], &v[3], &v[4])
            };

            format!(
                "```
UV Index => {}, {} (lat: {}, lon: {})

Current UV: {:.2} as of {}

Current Safe Exposure Times

Skin Type 1: {} min.
Skin Type 2: {} min.
Skin Type 3: {} min.

Max UV for Today: {:.2} at {}

Sun Times

Sunrise:    {}
Solar Noon: {}
Sunset:     {}
```",
                city,
                state,
                lat,
                lon,
                data.result.uv,
                uv_time,
                data.result.safe_exposure_time.st1.unwrap_or(0),
                data.result.safe_exposure_time.st2.unwrap_or(0),
                data.result.safe_exposure_time.st3.unwrap_or(0),
                data.result.uv_max,
                uv_max_time,
                sunrise,
                sun_noon,
                sun_set
            )
        }
        Err(_) => "`There was an error retrieving current data`".to_string(),
    }
}

#[command]
pub async fn current(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    match args.single::<i32>() {
        Ok(zip_code) => {
            let data = parse_current(zip_code).await;
            msg.channel_id.say(&ctx.http, data).await?
        }
        Err(_) => {
            msg.channel_id.say(&ctx.http, "`The zip code provided is invalid`".to_string()).await?
        }
    };

    Ok(())
}

pub async fn parse_forecast(zip_code: i32) -> String {
    let (city, state, lat, lon) = fetch_location(zip_code).await;

    match fetch_forecast(lat, lon).await {
        Ok(data) => {
            let v: Vec<_> = data.result.iter().map(|x| x.uv).collect();
            let v2: Vec<_> = data
                .result
                .iter()
                .map(|x| Local.from_utc_datetime(&x.uv_time.naive_local()).format("%I:%M %p"))
                .collect();
            let mut forecast = String::new();
            let combined = v.iter().zip(v2.iter());

            for (value, time) in combined {
                let entry = format!("{} - {:.2}\n", time, value);
                forecast.push_str(&entry)
            }

            format!(
                "```
UV Forecast => {}, {} (lat: {}, lon: {})

Forecast for {}

{}
```",
                city,
                state,
                lat,
                lon,
                Local.from_utc_datetime(&data.result[0].uv_time.naive_local()).format("%B %d, %Y"),
                forecast
            )
        }
        Err(_) => "`There was an error retrieving forecasting data`".to_string(),
    }
}

#[command]
pub async fn forecast(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    match args.single::<i32>() {
        Ok(zip_code) => {
            let data = parse_forecast(zip_code).await;
            msg.channel_id.say(&ctx.http, data).await?
        }
        Err(_) => {
            msg.channel_id.say(&ctx.http, "`The zip code provided is invalid`".to_string()).await?
        }
    };

    Ok(())
}
