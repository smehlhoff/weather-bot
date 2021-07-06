use crate::lib::config;
use crate::lib::error;

#[derive(Deserialize, Debug)]
pub struct CurrentResult {
    pub location: Location,
    pub current: Current,
}

#[derive(Deserialize, Debug)]
pub struct Location {
    pub name: String,
    pub country: String,
    pub region: String,
    pub lat: String,
    pub lon: String,
    pub timezone_id: String,
    pub localtime_epoch: i32,
    pub utc_offset: String,
}

#[derive(Deserialize, Debug)]
pub struct Current {
    pub observation_time: String,
    pub temperature: i32,
    pub weather_code: i32,
    pub weather_icons: Vec<String>,
    pub weather_descriptions: Vec<String>,
    pub wind_speed: i32,
    pub wind_degree: i32,
    pub wind_dir: String,
    pub pressure: i32,
    pub precip: i32,
    pub humidity: i32,
    pub cloudcover: i32,
    pub feelslike: i32,
    pub uv_index: i32,
    pub visibility: i32,
}

pub async fn fetch_current(zip_code: i32) -> Result<CurrentResult, error::Error> {
    let config = config::Config::load_config()?;
    let url = format!(
        "http://api.weatherstack.com/current?access_key={}&query={}",
        config.weatherstack, zip_code
    );
    let resp: CurrentResult = reqwest::get(&url).await?.json().await?;

    Ok(resp)
}

pub async fn parse_current(zip_code: i32) -> String {
    unimplemented!()
}
