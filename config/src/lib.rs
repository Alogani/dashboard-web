use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

mod user_config;
pub use user_config::*;
mod log_level;
use log_level::LogLevel;
mod route_check;

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AppConfig {
    pub log_level: LogLevel,
    pub cookie_domain: String,
    pub router_address: String,
    pub users_file: String,
    allowed_routes: HashMap<String, Vec<String>>,
    allowed_subdomains: HashMap<String, Vec<String>>,
    pub external_links: HashMap<String, String>,
}

impl AppConfig {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;

        let deserializer = toml::de::Deserializer::new(&content);

        let config = AppConfig::deserialize(deserializer)
            .map_err(|e| format!("Failed to parse config file: {}", e))?;

        Ok(config)
    }
}
