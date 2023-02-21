use secrecy::Secret;
use serde::Deserialize;
use std::{env, path::PathBuf};
use tracing::error;

const CONFIG_PATH_ENV: &str = "CONFIG_PATH";

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    /// Path to service account credentials that have access to spreadsheet that sheet_id points
    /// to. To give service account an access to spreadsheet you need to share it to the service
    /// account email like you would share it to a real user.
    pub service_account_credentials_path: PathBuf,
    /// Spreadsheet id you can get from the browser URL
    pub sheet_id: String,
    pub vertical_life_email: String,
    pub vertical_life_password: Secret<String>,
    /// Vertical life gym ids that should be fetched. Spreadsheet should have matching sheet(s)
    /// with name for example 'Ristikko - Reitit'. The program will be able to tell that for gym
    /// id 2108 it needs to climbs to that tab (or the bouldering equivalent).
    pub gyms: Vec<u32>,
    pub climb_color_column_idx: i32,
    pub grade_column_idx: i32,
    pub date_column_idx: i32,
    pub new_climb_background_color: String,
}

pub fn read_config() -> Config {
    env::var(CONFIG_PATH_ENV)
        .map_err(|_| format!("{CONFIG_PATH_ENV} environment variable not set"))
        .and_then(|config_path| std::fs::read_to_string(config_path).map_err(|e| e.to_string()))
        .and_then(|str| toml::from_str(&str).map_err(|e| e.to_string()))
        .unwrap_or_else(|err| {
            error!("failed to read config: {err}");
            std::process::exit(1);
        })
}
