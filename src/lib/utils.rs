use serenity::framework::standard::Args;
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::error::Error;
use crate::lib::db;
use crate::Database;

#[derive(Debug, Deserialize)]
pub struct GeocodeResponse {
    pub results: Vec<GeocodeData>,
}

#[derive(Debug, Deserialize)]
pub struct GeocodeData {
    pub name: String,
    pub admin1: String,
    pub latitude: f64,
    pub longitude: f64,
}

pub fn check_zip_code(arg: &str) -> Result<i32, Error> {
    if arg.len() == 5 {
        match arg.parse::<i32>() {
            Ok(zip_code) => Ok(zip_code),
            Err(_) => Err(Error::Invalid("The zip code provided is invalid".into())),
        }
    } else {
        Err(Error::Invalid("The zip code provided is not five digits".into()))
    }
}

pub fn check_station_code(station: &str) -> Result<(), Error> {
    if station.len() == 4 {
        if station.starts_with('K') {
            Ok(())
        } else {
            Err(Error::Invalid("U.S. station codes only (e.g., KSFO, KJFK)".into()))
        }
    } else {
        Err(Error::Invalid("The station code provided is invalid".into()))
    }
}

pub async fn check_location(ctx: &Context, msg: &Message, args: &Args) -> Result<String, Error> {
    if args.message().is_empty() {
        let pool = {
            let data = ctx.data.read().await;
            data.get::<Database>().expect("Error retrieving database pool").clone()
        };
        let zip_code = db::fetch_location(&pool, msg).await?;

        Ok(zip_code)
    } else {
        Ok(String::from(args.message()))
    }
}

pub async fn fetch_location(zip_code: i32) -> Result<GeocodeResponse, Error> {
    let url = format!("https://geocoding-api.open-meteo.com/v1/search?name={zip_code}&count=1&language=en&format=json");
    let resp = reqwest::get(&url).await?.json().await;

    match resp {
        Ok(data) => {
            let resp: GeocodeResponse = data;
            Ok(resp)
        }
        Err(_) => Err(Error::NotFound("The zip code provided does not match a location".into())),
    }
}

pub fn cardinal_direction(val: &str) -> String {
    let val: f64 = val.parse::<f64>().unwrap();
    let directions = [
        "N", "NNE", "NE", "ENE", "E", "ESE", "SE", "SSE", "S", "SSW", "SW", "WSW", "W", "WNW",
        "NW", "NNW", "N",
    ];
    let index = (val / 22.5).round() as usize;

    String::from(directions[index])
}
