use serenity::framework::standard::{macros::command, CommandResult};
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

async fn fetch_alerts() -> Result<(AlertResponse, String), Error> {
    let config = config::Config::load_config()?;

    if config.alert_zone.is_empty() {
        Err(Error::NotFound("The config file does not have an alert zone value".into()))
    } else {
        let url = format!("https://api.weather.gov/alerts/active/zone/{}", config.alert_zone);
        let client = reqwest::ClientBuilder::new().user_agent(config.user_agent).build()?;
        let resp = client.get(&url).send().await?.json().await;

        match resp {
            Ok(data) => {
                let resp: AlertResponse = data;
                Ok((resp, config.alert_zone))
            }
            Err(_) => Err(Error::NotFound("The alert zone provided does not exist".into())),
        }
    }
}

async fn parse_alerts() -> String {
    match fetch_alerts().await {
        Ok((data, zone)) => {
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
                format!("```{}\n\n{}\nRead more here: https://alerts.weather.gov/cap/wwaatmget.php?x={}&y=1```", data.title, alerts, zone)
            }
        }
        Err(e) => format!("`There was an error retrieving alerts: {e}`"),
    }
}

#[command]
pub async fn alerts(ctx: &Context, msg: &Message) -> CommandResult {
    let data = parse_alerts().await;

    msg.channel_id.say(&ctx.http, data).await?;

    Ok(())
}
