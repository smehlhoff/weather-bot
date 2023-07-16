use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::lib::{config, error::Error};

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct AlertResponse {
    features: Vec<AlertFeature>,
    title: String,
    updated: String,
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

pub async fn parse_alerts(alert_zone: &str) -> String {
    match fetch_alerts(alert_zone).await {
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
        Err(e) => format!("`There was an error retrieving alerts: {e}`"),
    }
}

#[command]
pub async fn alerts(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let args: Vec<&str> = args.message().split(' ').collect();

    for arg in args {
        let data = parse_alerts(arg).await;
        msg.channel_id.say(&ctx.http, data).await?;
    }

    Ok(())
}
