use bcrypt::{DEFAULT_COST, hash, verify};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

const AUTH_SALT: &str = "h0m3w3bs3rv3r_s4lt_f0r_4uth";

#[derive(Debug, Deserialize, Serialize)]
pub struct User {
    pub username: String,
    pub password_hash: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UsersConfig {
    pub users: HashMap<String, String>,
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
