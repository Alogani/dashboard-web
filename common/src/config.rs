use bcrypt::{hash, verify, DEFAULT_COST};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

const AUTH_SALT: &str = "h0m3w3bs3rv3r_s4lt_f0r_4uth";

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AppConfig {
    pub log_level: LogLevel,
    pub cookie_domain: String,
    pub router_address: String,
    pub users_file: String,
    allowed_routes: HashMap<String, Vec<String>>,
    allowed_subdomains: HashMap<String, Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct User {
    pub username: String,
    pub password_hash: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UsersConfig {
    pub users: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl AppConfig {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;

        let deserializer = toml::de::Deserializer::new(&content);

        let config = AppConfig::deserialize(deserializer)
            .map_err(|e| format!("Failed to parse config file: {}", e))?;

        Ok(config)
    }

    pub fn is_route_allowed(&self, route: &str, username: Option<&str>) -> bool {
        // Special case for root path "/"
        if let Some(allowed_users) = self.allowed_routes.get("/") {
            // Root path should not automatically grant access to all paths
            // It should only match exactly "/"
            if route == "/" {
                if let Some(username) = username {
                    return allowed_users.contains(&username.to_string())
                        || allowed_users.contains(&"*".to_string());
                } else {
                    return allowed_users.contains(&"*".to_string());
                }
            }
        }

        // Then, check for exact matches which take precedence
        if let Some(allowed_users) = self.allowed_routes.get(route) {
            // If username is None, only allow access if "*" is in allowed_users
            if let Some(username) = username {
                if allowed_users.contains(&username.to_string())
                    || allowed_users.contains(&"*".to_string())
                {
                    return true;
                }
            } else if allowed_users.contains(&"*".to_string()) {
                return true;
            }
        }

        // Then check for parent paths with proper path segment boundaries
        // Sort paths by length in descending order to check most specific paths first
        let mut paths: Vec<(&String, &Vec<String>)> = self.allowed_routes.iter().collect();
        paths.sort_by(|(a, _), (b, _)| b.len().cmp(&a.len()));

        for (path, allowed_users) in paths {
            // Skip exact match as we already checked it
            if path == route {
                continue;
            }

            // Check if this is a parent path with proper path boundary
            if path.ends_with('/') && route.starts_with(path) {
                // Path ends with slash, so it's a directory-like path
                if let Some(username) = username {
                    if allowed_users.contains(&username.to_string())
                        || allowed_users.contains(&"*".to_string())
                    {
                        return true;
                    }
                } else if allowed_users.contains(&"*".to_string()) {
                    return true;
                }
            } else if !path.is_empty() && route.starts_with(&format!("{}/", path)) {
                // Path doesn't end with slash, ensure we're checking a proper subdirectory
                // with a path separator and that path is not empty
                if let Some(username) = username {
                    if allowed_users.contains(&username.to_string())
                        || allowed_users.contains(&"*".to_string())
                    {
                        return true;
                    }
                } else if allowed_users.contains(&"*".to_string()) {
                    return true;
                }
            }
        }

        false
    }

    pub fn is_subdomain_allowed(&self, subdomain: &str, username: Option<&str>) -> bool {
        if let Some(allowed_users) = self.allowed_subdomains.get(subdomain) {
            // If username is None, only allow access if "*" is in allowed_users
            if let Some(username) = username {
                return allowed_users.contains(&username.to_string())
                    || allowed_users.contains(&"*".to_string());
            } else {
                return allowed_users.contains(&"*".to_string());
            }
        }
        false
    }
}

impl UsersConfig {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let mut users = HashMap::new();

        // Parse the simple format: username = password_hash
        for line in content.lines() {
            if line.trim().is_empty() || line.trim().starts_with('#') {
                continue; // Skip empty lines and comments
            }

            if let Some((username, hash)) = line.split_once('=') {
                users.insert(username.trim().to_string(), hash.trim().to_string());
            }
        }

        Ok(UsersConfig { users })
    }

    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let mut content = String::new();

        for (username, hash) in &self.users {
            content.push_str(&format!("{} = {}\n", username, hash));
        }

        fs::write(path, content)?;
        Ok(())
    }

    pub fn get_password_hash(&self, username: &str) -> Option<&str> {
        self.users.get(username).map(|hash| hash.as_str())
    }

    pub fn get_user_hash(&self, username: &str) -> Option<String> {
        if let Some(password_hash) = self.get_password_hash(username) {
            // Create a hash from username + password_hash + salt
            let auth_string = format!("{}{}{}", username, password_hash, AUTH_SALT);

            // Use bcrypt instead of SHA-256
            match hash(&auth_string, DEFAULT_COST) {
                Ok(hashed) => Some(hashed),
                Err(_) => None,
            }
        } else {
            None
        }
    }

    // Function to get username from an auth hash
    pub fn get_username_from_hash(&self, auth_hash: &str) -> Option<String> {
        // Check each user to see if their auth hash matches
        for username in self.users.keys() {
            if let Some(password_hash) = self.get_password_hash(username) {
                // Create the auth string
                let auth_string = format!("{}{}{}", username, password_hash, AUTH_SALT);

                // Use bcrypt verify to check if the hash matches
                if let Ok(true) = verify(&auth_string, auth_hash) {
                    return Some(username.clone());
                }
            }
        }
        None
    }
}

impl ToString for LogLevel {
    fn to_string(&self) -> String {
        match self {
            LogLevel::Trace => "trace".to_string(),
            LogLevel::Debug => "debug".to_string(),
            LogLevel::Info => "info".to_string(),
            LogLevel::Warn => "warn".to_string(),
            LogLevel::Error => "error".to_string(),
        }
    }
}
