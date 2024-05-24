use std::fs;

use crate::lib::error;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub admin: u64,
    pub alert_users: Vec<u64>,
    pub alert_zip_codes: Vec<i32>,
    pub debug: bool,
    pub discord: String,
    pub openuv: String,
    pub user_agent: String,
    pub uv_users: Vec<u64>,
    pub uv_zip_codes: Vec<i32>,
}

impl Config {
    pub fn load_config() -> Result<Self, error::Error> {
        let file = fs::OpenOptions::new().read(true).open("config.json")?;
        let json: Self = serde_json::from_reader(file)?;

        Ok(json)
    }
}
