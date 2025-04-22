use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::net::Ipv4Addr;
use std::path::Path;

mod user_config;
pub use user_config::*;
mod log_level;
use log_level::LogLevel;
mod route_check;

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AppConfig {
    #[serde(default = "default_server_address")]
    server_address: Ipv4Addr,
    #[serde(default = "default_server_port")]
    server_port: u16,
    log_level: LogLevel,
    cookie_domain: String,
    router_address: String,
    users_file: String,
    allowed_routes: HashMap<String, Vec<String>>,
    allowed_subdomains: HashMap<String, Vec<String>>,
    external_links: HashMap<String, String>,
}

fn default_server_address() -> Ipv4Addr {
    Ipv4Addr::new(127, 0, 0, 1)
}
fn default_server_port() -> u16 {
    8080
}

impl AppConfig {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;

        let deserializer = toml::de::Deserializer::new(&content);

        let config = AppConfig::deserialize(deserializer)
            .map_err(|e| format!("Failed to parse config file: {}", e))?;

        Ok(config)
    }

    pub fn get_server_address(&self) -> Ipv4Addr {
        self.server_address
    }

    pub fn get_server_port(&self) -> u16 {
        self.server_port
    }

    pub fn get_log_level(&self) -> LogLevel {
        self.log_level
    }

    pub fn get_cookie_domain(&self) -> &str {
        &self.cookie_domain
    }

    pub fn get_router_address(&self) -> &str {
        &self.router_address
    }

    pub fn get_users_file(&self) -> &str {
        &self.users_file
    }

    pub fn get_external_links(&self) -> &HashMap<String, String> {
        &self.external_links
    }
}
