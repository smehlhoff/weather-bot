use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::commands::wx;
use crate::lib::{config, error::Error, utils, utils::GeocodeResponse};

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct AlertResponse {
    features: Vec<AlertFeature>,
    title: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct AlertFeature {
    properties: AlertProperties,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct AlertProperties {
    headline: String,
    severity: String,
}

async fn fetch_alerts(alert_zone: &str) -> Result<AlertResponse, Error> {
    let config = config::Config::load_config()?;
    let alert_zone = alert_zone.to_uppercase();
    let url = format!("https://api.weather.gov/alerts/active/zone/{alert_zone}");
    let client = reqwest::ClientBuilder::new().user_agent(config.user_agent).build()?;
    let resp = client.get(&url).send().await?.json().await;

    match resp {
        Ok(data) => {
            let resp: AlertResponse = data;
            Ok(resp)
        }
        Err(_) => Err(Error::NotFound("The alert zone provided does not exist".into())),
    }
}

pub async fn parse_alerts(data: GeocodeResponse) -> String {
    let (lat, lon) = (data.results[0].latitude, data.results[0].longitude);

    match wx::fetch_wx(lat, lon).await {
        Ok(data) => {
            let alert_zone = data.location.zone;
            match fetch_alerts(&alert_zone).await {
                Ok(data) => {
                    if data.features.is_empty() {
                        ("`No active alerts for this zone`").to_string()
                    } else {
                        let mut alerts = String::new();

                        for alert in data.features.iter().rev() {
                            alerts.push_str(&format!(
                                "- {} ({})\n",
                                alert.properties.headline, alert.properties.severity,
                            ));
                        }
                        format!("```{}\n\n{}\nRead more here: https://alerts.weather.gov/cap/wwaatmget.php?x={}&y=1```", data.title, alerts, alert_zone)
                    }
                }
                Err(e) => format!("`There was an error retrieving data: {e}`"),
            }
        }
        Err(e) => format!("`There was an error retrieving data: {e}`"),
    }
}

#[command]
pub async fn alerts(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let args = match utils::check_location(ctx, msg, &args).await {
        Ok(val) => val,
        Err(_) => String::new(),
    };
    let args: Vec<&str> = args.split(' ').collect();

    for arg in args {
        match utils::check_zip_code(arg) {
            Ok(zip_code) => match utils::fetch_location(zip_code).await {
                Ok(data) => {
                    let data = parse_alerts(data).await;
                    msg.channel_id.say(&ctx.http, data).await?
                }
                Err(e) => msg.channel_id.say(&ctx.http, format!("`{e}`")).await?,
            },
            Err(e) => msg.channel_id.say(&ctx.http, format!("`{e}`")).await?,
        };
    }

    Ok(())
}
