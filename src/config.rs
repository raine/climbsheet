use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;
use std::{env, path::PathBuf};
use tracing::error;

const CONFIG_PATH_ENV: &str = "CONFIG_PATH";

#[derive(Debug, Deserialize)]
pub struct SecretString(Secret<String>);

impl SecretString {
    pub fn expose_secret(&self) -> &str {
        self.0.expose_secret()
    }
}

impl Default for SecretString {
    fn default() -> Self {
        SecretString(Secret::new("".to_string()))
    }
}

#[derive(Deserialize, Debug, Default)]
pub struct Config {
    pub service_account_credentials_path: PathBuf,
    pub sheet_id: String,
    pub vertical_life_email: String,
    pub vertical_life_password: SecretString,
    pub gyms: Vec<u32>,
    pub climb_color_column_idx: i32,
    pub date_column_idx: i32,
}

pub fn read_config() -> Config {
    env::var(&CONFIG_PATH_ENV)
        .map_err(|_| format!("{CONFIG_PATH_ENV} environment variable not set"))
        .and_then(|config_path| std::fs::read_to_string(config_path).map_err(|e| e.to_string()))
        .and_then(|str| toml::from_str(&str).map_err(|e| e.to_string()))
        .unwrap_or_else(|err| {
            error!("failed to read config: {err}");
            std::process::exit(1);
        })
}
