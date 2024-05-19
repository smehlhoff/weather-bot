use serenity::framework::standard::Args;
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::error::Error;
use crate::lib::db;
use crate::Database;

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
        Ok(args.message().to_string())
    }
}
