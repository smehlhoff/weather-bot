#![warn(clippy::all)]
// #![warn(clippy::nursery)]
// #![warn(clippy::pedantic)]

#[macro_use]
extern crate serde;

use chrono::{DateTime, Local, NaiveTime};
use once_cell::sync::OnceCell;
use serenity::{
    async_trait,
    framework::standard::{macros::group, StandardFramework},
    model::{channel::Message, gateway::Ready, id::UserId},
    prelude::*,
};
use sqlx::Sqlite;
use std::fs;

use std::time;

mod commands {
    pub mod alerts;
    pub mod atis;
    pub mod location;
    pub mod meta;
    pub mod metar;
    pub mod taf;
    pub mod uv;
    pub mod wx;
}

mod lib {
    pub mod config;
    pub mod db;
    pub mod error;
    pub mod utils;
}

#[allow(clippy::wildcard_imports)]
use commands::{alerts::*, atis::*, location::*, meta::*, metar::*, taf::*, uv::*, wx::*};
use lib::{config, db, error, utils};

static CELL: OnceCell<()> = OnceCell::new();

struct Handler;

impl Handler {
    async fn run_background_tasks(ctx: &Context) -> Result<(), error::Error> {
        let config = config::Config::load_config()?;
        let start_time = NaiveTime::from_hms_opt(8, 30, 0).unwrap();
        let end_time = NaiveTime::from_hms_opt(8, 31, 0).unwrap();
        let current_time = Local::now().time();

        if (current_time >= start_time)
            && (current_time < end_time)
            && !config.alert_zip_codes.is_empty()
        {
            for zip_code in config.alert_zip_codes {
                let alert_zone =
                    utils::fetch_location(zip_code).await.expect("Error retrieving alert zone");
                let data = commands::alerts::parse_alerts(zip_code, alert_zone).await;
                for user in &config.alert_users {
                    if let Err(e) = Self::message_user(ctx, *user, &data).await {
                        println!("Error sending message to user: {e}");
                    }
                    tokio::time::sleep(time::Duration::from_secs(3)).await;
                }
            }
        }

        if (current_time >= start_time)
            && (current_time < end_time)
            && !config.uv_zip_codes.is_empty()
        {
            for zip_code in config.uv_zip_codes {
                let data = commands::uv::parse_forecast(zip_code).await;
                for user in &config.uv_users {
                    if let Err(e) = Self::message_user(ctx, *user, &data).await {
                        println!("Error sending message to user: {e}");
                    }
                    tokio::time::sleep(time::Duration::from_secs(3)).await;
                }
            }
        }

        Ok(())
    }

    async fn healthcheck() -> Result<(), error::Error> {
        let config = config::Config::load_config()?;

        if !config.healthcheck.is_empty() {
            let resp = reqwest::get(&config.healthcheck).await;

            match resp {
                Ok(_) => {}
                Err(e) => {
                    println!("Error sending healthcheck: {e}")
                }
            }
        }

        Ok(())
    }

    async fn message_user(ctx: &Context, user: u64, data: &str) -> Result<(), error::Error> {
        UserId(user)
            .create_dm_channel(&ctx.http)
            .await?
            .send_message(&ctx.http, |m| m.content(data))
            .await?;

        Ok(())
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected.", ready.user.name);

        CELL.get_or_init(|| {
            tokio::spawn(async move {
                loop {
                    Self::run_background_tasks(&ctx).await.expect("Error running background tasks");
                    tokio::time::sleep(time::Duration::from_secs(60)).await;
                }
            });

            tokio::spawn(async {
                loop {
                    Self::healthcheck().await.expect("Error running healthcheck background");
                    tokio::time::sleep(time::Duration::from_secs(60)).await;
                }
            });
        });

        tokio::spawn(async {
            loop {
                if fs::metadata("./attachments").is_ok() {
                    match fs::remove_dir_all("./attachments") {
                        Ok(()) => {}
                        Err(e) => println!("Error deleting directory: {e}"),
                    }
                }
                fs::create_dir_all("./attachments")
                    .expect("Error creating ./attachments directory");

                tokio::time::sleep(time::Duration::from_secs(86400)).await;
            }
        });
    }

    async fn message(&self, ctx: Context, msg: Message) {
        if ctx.cache.current_user_id() != msg.author.id.0 {
            tokio::spawn(async move {
                let pool = {
                    let data = ctx.data.read().await;
                    data.get::<Database>().expect("Error retrieving database pool").clone()
                };

                db::insert_log(&pool, msg).await
            });
        }
    }
}

struct BotAdmin;

impl TypeMapKey for BotAdmin {
    type Value = u64;
}

struct Database;

impl TypeMapKey for Database {
    type Value = sqlx::Pool<Sqlite>;
}

struct Uptime;

impl TypeMapKey for Uptime {
    type Value = DateTime<Local>;
}

#[group]
struct Admin;

#[group]
#[commands(alerts)]
struct Alerts;

#[group]
#[commands(atis)]
struct Atis;

#[group]
#[prefixes("location")]
#[commands(location_set, location_list, location_delete)]
struct Location;

#[group]
#[commands(ping, utc, uptime, logs, help)]
struct Meta;

#[group]
#[commands(metar)]
struct METAR;

#[group]
#[commands(taf)]
struct TAF;

#[group]
#[prefixes("uv")]
#[commands(uv_current, uv_forecast)]
struct UV;

#[group]
#[prefixes("wx")]
#[commands(wx_current, wx_forecast, wx_graph)]
struct WX;

#[tokio::main]
async fn main() {
    let config = config::Config::load_config().expect("Error loading config file");
    let prefix = {
        if config.debug {
            "?"
        } else {
            "!"
        }
    };
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename("db.sqlite3")
                .create_if_missing(true),
        )
        .await
        .expect("Error connecting to database");
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;
    let framework = StandardFramework::new()
        .configure(|c| c.prefix(prefix))
        .group(&ADMIN_GROUP)
        .group(&ALERTS_GROUP)
        .group(&ATIS_GROUP)
        .group(&LOCATION_GROUP)
        .group(&META_GROUP)
        .group(&METAR_GROUP)
        .group(&TAF_GROUP)
        .group(&UV_GROUP)
        .group(&WX_GROUP);
    let mut client = Client::builder(&config.discord, intents)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");

    db::create_log_table(&pool).await.expect("Error creating database table");

    {
        let mut data = client.data.write().await;
        data.insert::<BotAdmin>(config.admin);
    }

    {
        let mut data = client.data.write().await;
        data.insert::<Database>(pool);
    }

    {
        let mut data = client.data.write().await;
        data.insert::<Uptime>(Local::now());
    }

    if let Err(e) = client.start().await {
        panic!("Error connecting client: {}", e);
    }
}
