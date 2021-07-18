#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]

#[macro_use]
extern crate serde;

use chrono::{DateTime, Local, NaiveTime};
use serenity::{
    async_trait,
    framework::standard::{macros::group, StandardFramework},
    model::{gateway::Ready, id::UserId},
    prelude::*,
};

use std::time;

mod commands;
mod lib;

use commands::meta::*;
use commands::metar::*;
use commands::uv::*;
use commands::wx::*;

use lib::config;
use lib::error;

struct Handler;

impl Handler {
    async fn message_user(ctx: &Context, user: u64, data: &str) -> Result<(), error::Error> {
        UserId(user)
            .create_dm_channel(&ctx.http)
            .await?
            .send_message(&ctx.http, |m| m.content(data))
            .await?;

        Ok(())
    }

    async fn uv_background(ctx: &Context) -> Result<(), error::Error> {
        let config = config::Config::load_config()?;
        let start_time = NaiveTime::from_hms(8, 0, 0);
        let end_time = NaiveTime::from_hms(8, 1, 0);
        let current_time = Local::now().time();

        if (current_time >= start_time) && (current_time <= end_time) {
            for zip_code in config.zip_codes {
                let data = commands::uv::parse_forecast(zip_code).await;
                for user in &config.users {
                    if let Err(e) = Self::message_user(ctx, *user, &data).await {
                        println!("Unable to message user: {}", e)
                    }
                }
            }
        }

        Ok(())
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected.", ready.user.name);

        tokio::spawn(async move {
            loop {
                Self::uv_background(&ctx).await.unwrap();
                tokio::time::sleep(time::Duration::from_secs(60)).await;
            }
        });
    }
}

struct Uptime;

impl TypeMapKey for Uptime {
    type Value = DateTime<Local>;
}

#[group]
struct Admin;

#[group]
#[commands(ping, uptime, help)]
struct Meta;

#[group]
#[commands(metar)]
struct METAR;

#[group]
#[prefixes("uv")]
#[commands(current, forecast)]
struct UV;

#[group]
#[commands(wx)]
struct WX;

#[tokio::main]
async fn main() {
    let config = config::Config::load_config().expect("Unable to load config file");
    let framework = StandardFramework::new()
        .configure(|c| c.prefix("!"))
        .group(&ADMIN_GROUP)
        .group(&META_GROUP)
        .group(&METAR_GROUP)
        .group(&UV_GROUP)
        .group(&WX_GROUP);
    let mut client = Client::builder(&config.discord)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Unable to create client");

    {
        let mut data = client.data.write().await;
        data.insert::<Uptime>(Local::now());
    }

    if let Err(e) = client.start().await {
        panic!("Error connecting client: {}", e)
    }
}
