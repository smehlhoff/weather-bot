use std::fs;

use crate::lib::error;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub discord: String,
    pub users: Vec<u64>,
    pub openuv: String,
    pub weatherstack: String,
    pub zip_codes: Vec<i32>,
}

impl Config {
    pub fn load_config() -> Result<Self, error::Error> {
        let file = fs::OpenOptions::new().read(true).open("config.json")?;
        let json: Self = serde_json::from_reader(file)?;

        Ok(json)
    }
}
