pub mod admin_config;
mod user_config;
use access_check::access_rules_deserialize;
pub use user_config::*;
mod log_level;
use log_level::LogLevel;
mod access_check;

use admin_config::AdminConsole;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::net::Ipv4Addr;
use std::path::Path;
use utils::string_tuple_vec;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AppConfig {
    #[serde(default = "default_server_address")]
    server_address: Ipv4Addr,
    #[serde(default = "default_server_port")]
    server_port: u16,
    static_folder: String,
    log_level: LogLevel,
    cookie_domain: String,
    users_file: String,
    secure_cookies: bool,
    cookie_duration: u32,
    #[serde(deserialize_with = "access_rules_deserialize")]
    access_rules: HashMap<String, Vec<(String, Vec<String>)>>,
    #[serde(with = "string_tuple_vec")]
    external_links: Vec<(String, String)>,
    admin_commands: AdminConsole,
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

    pub fn get_admin_commands(&self) -> &AdminConsole {
        &self.admin_commands
    }

    pub fn get_cookie_domain(&self) -> &str {
        &self.cookie_domain
    }

    pub fn get_cookie_duration(&self) -> u32 {
        self.cookie_duration
    }

    pub fn get_external_links(&self) -> &Vec<(String, String)> {
        &self.external_links
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

    pub fn get_static_folder(&self) -> &str {
        &self.static_folder
    }

    pub fn get_users_file(&self) -> &str {
        &self.users_file
    }

    pub fn use_secure_cookies(&self) -> bool {
        self.secure_cookies
    }
}
