use chrono::prelude::*;
use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::*,
    prelude::*,
};

use crate::lib::{config, error::Error, utils};

#[derive(Debug, Deserialize)]
struct CurrentResult {
    result: UVCurrent,
}

#[derive(Debug, Deserialize)]
struct UVCurrent {
    uv: f64,
    uv_time: chrono::DateTime<Utc>,
    uv_max: f64,
    uv_max_time: chrono::DateTime<Utc>,
    safe_exposure_time: SafeExposureTime,
    sun_info: SunInfo,
}

#[derive(Debug, Deserialize)]
struct SafeExposureTime {
    st1: Option<i32>,
    st2: Option<i32>,
    st3: Option<i32>,
}

#[derive(Debug, Deserialize)]
struct SunInfo {
    sun_times: SunTimes,
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize)]
struct SunTimes {
    sunrise: chrono::DateTime<Utc>,
    solarNoon: chrono::DateTime<Utc>,
    sunset: chrono::DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
struct ForecastResult {
    result: Vec<ForecastPeriod>,
}

#[derive(Debug, Deserialize)]
struct ForecastPeriod {
    uv: f64,
    uv_time: chrono::DateTime<Utc>,
}

async fn fetch_current(lat: f64, lon: f64) -> Result<CurrentResult, Error> {
    let config = config::Config::load_config()?;
    let url = format!("https://api.openuv.io/api/v1/uv?lat={lat}&lng={lon}");
    let client = reqwest::Client::new();
    let resp = client.get(&url).header("x-access-token", config.openuv).send().await?.json().await;

    match resp {
        Ok(data) => {
            let resp: CurrentResult = data;
            Ok(resp)
        }
        Err(_) => Err(Error::NotFound("The zip code provided does not match a location".into())),
    }
}
async fn fetch_forecast(lat: f64, lon: f64) -> Result<ForecastResult, Error> {
    let config = config::Config::load_config()?;
    let url = format!("https://api.openuv.io/api/v1/forecast?lat={lat}&lng={lon}");
    let client = reqwest::Client::new();
    let resp = client.get(&url).header("x-access-token", config.openuv).send().await?.json().await;

    match resp {
        Ok(data) => {
            let resp: ForecastResult = data;
            Ok(resp)
        }
        Err(_) => Err(Error::NotFound("The zip code provided does not match a location".into())),
    }
}

async fn parse_current(zip_code: i32) -> String {
    match utils::fetch_location(zip_code).await {
        Ok(data) => {
            let (city, state, lat, lon) = (
                &data.results[0].name,
                &data.results[0].admin1,
                data.results[0].latitude,
                data.results[0].longitude,
            );
            match fetch_current(lat, lon).await {
                Ok(data) => {
                    #[allow(unused_assignments)]
                    let mut v = Vec::new();
                    let (uv_time, uv_max_time, sunrise, sun_noon, sun_set) = {
                        let v2 = [
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
UV Index => {}, {} (lat: {:.2}, lon: {:.2})

Current UV: {:.2}

Current Safe Exposure Times
---------------------------

Skin Type 1:    {} min.
Skin Type 2:    {} min.
Skin Type 3:    {} min.

Max UV for Today: {:.2} at {}

Sun Times
---------

Sunrise:        {}
Solar Noon:     {}
Sunset:         {}

Last updated at {}
```",
                        city,
                        state,
                        lat,
                        lon,
                        data.result.uv,
                        data.result.safe_exposure_time.st1.unwrap_or(0),
                        data.result.safe_exposure_time.st2.unwrap_or(0),
                        data.result.safe_exposure_time.st3.unwrap_or(0),
                        data.result.uv_max,
                        uv_max_time,
                        sunrise,
                        sun_noon,
                        sun_set,
                        uv_time
                    )
                }
                Err(e) => format!("`There was an error retrieving data: {e}`"),
            }
        }
        Err(e) => format!("`There was an error retrieving data: {e}`"),
    }
}

#[command]
#[aliases("current")]
pub async fn uv_current(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let args = match utils::check_location(ctx, msg, &args).await {
        Ok(val) => val,
        Err(_) => String::new(),
    };
    let args: Vec<&str> = args.split(' ').collect();

    for arg in args {
        match utils::check_zip_code(arg) {
            Ok(zip_code) => {
                let data = parse_current(zip_code).await;
                msg.channel_id.say(&ctx.http, data).await?
            }
            Err(e) => msg.channel_id.say(&ctx.http, format!("`{e}`")).await?,
        };
    }

    Ok(())
}

pub async fn parse_forecast(zip_code: i32) -> String {
    match utils::fetch_location(zip_code).await {
        Ok(data) => {
            let (city, state, lat, lon) = (
                &data.results[0].name,
                &data.results[0].admin1,
                data.results[0].latitude,
                data.results[0].longitude,
            );
            match fetch_forecast(lat, lon).await {
                Ok(data) => {
                    let v: Vec<f64> = data.result.iter().map(|x| x.uv).collect();
                    let v2: Vec<_> = data
                        .result
                        .iter()
                        .map(|x| {
                            Local.from_utc_datetime(&x.uv_time.naive_local()).format("%I:%M %p")
                        })
                        .collect();
                    let mut forecast = String::new();
                    let combined = v.iter().zip(v2.iter());

                    for (val, time) in combined {
                        let entry = format!("{time}: {val:.2}\n");
                        forecast.push_str(&entry);
                    }

                    format!(
                        "```
UV Forecast => {}, {} (lat: {:.2}, lon: {:.2})

Forecast for {}:

{}
```",
                        city,
                        state,
                        lat,
                        lon,
                        Local
                            .from_utc_datetime(&data.result[0].uv_time.naive_local())
                            .format("%B %d, %Y"),
                        forecast
                    )
                }
                Err(e) => format!("`There was an error retrieving data: {e}`"),
            }
        }
        Err(e) => format!("`There was an error retrieving data: {e}`"),
    }
}

#[command]
#[aliases("forecast")]
pub async fn uv_forecast(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let args = match utils::check_location(ctx, msg, &args).await {
        Ok(val) => val,
        Err(_) => String::new(),
    };
    let args: Vec<&str> = args.split(' ').collect();

    for arg in args {
        match utils::check_zip_code(arg) {
            Ok(zip_code) => {
                let data = parse_forecast(zip_code).await;
                msg.channel_id.say(&ctx.http, data).await?
            }
            Err(e) => msg.channel_id.say(&ctx.http, format!("`{e}`")).await?,
        };
    }

    Ok(())
}
