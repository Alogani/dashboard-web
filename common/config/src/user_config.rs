use bcrypt::{DEFAULT_COST, hash, verify};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use app_errors::AppError;

#[derive(Debug, Deserialize, Serialize)]
pub struct UsersConfig {
    #[serde(skip)]
    common_salt: String,
    // Username alongside the hash of their password
    #[serde(flatten)]
    users_private: HashMap<String, String>,
    // Username alongside a public hash of their password with a common salt
    #[serde(skip)]
    users_public: HashMap<String, String>,
    // Reverse lookup: public hash to username
    #[serde(skip)]
    public_hash_to_username: HashMap<String, String>,
}

fn generate_random_salt() -> String {
    let bytes = rand::rng()
        .sample_iter(&rand::distr::Alphanumeric)
        .take(16)
        .collect::<Vec<u8>>();
    String::from_utf8_lossy(&bytes).to_string()
}

fn compute_private_hash(password: &str) -> Result<String, AppError> {
    let private_hash = hash(password, DEFAULT_COST)?;
    Ok(private_hash)
}

fn compute_public_hash(username: &str, private_hash: &str, salt: &str) -> Result<String, AppError> {
    let public_hash = hash(&format!("{username}{private_hash}{salt}"), DEFAULT_COST)?;
    Ok(public_hash)
}

impl UsersConfig {
    pub fn new() -> Self {
        UsersConfig {
            common_salt: generate_random_salt(),
            users_private: HashMap::new(),
            users_public: HashMap::new(),
            public_hash_to_username: HashMap::new(),
        }
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, AppError> {
        let content = fs::read_to_string(path)?;
        let mut config: UsersConfig = toml::from_str(&content)?;

        // Generate a common salt if it doesn't exist
        if config.common_salt.is_empty() {
            config.common_salt = generate_random_salt();
        }

        // Compute common_salt_hash for each user and build reverse lookup
        for (username, password_hash) in &config.users_private {
            let public_hash = compute_public_hash(username, &password_hash, &config.common_salt)?;
            config
                .users_public
                .insert(username.clone(), public_hash.clone());
            config
                .public_hash_to_username
                .insert(public_hash, username.clone());
        }

        Ok(config)
    }

    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), AppError> {
        let serialized = toml::to_string(&self)?;
        fs::write(path, serialized)?;
        Ok(())
    }

    pub fn add_or_update_user(&mut self, username: String, password: &str) -> Result<(), AppError> {
        let private_hash = compute_private_hash(password)?;
        let public_hash = compute_public_hash(&username, &private_hash, &self.common_salt)?;

        // Remove old public hash from reverse lookup if it exists
        if let Some(old_public_hash) = self.users_public.get(&username) {
            self.public_hash_to_username.remove(old_public_hash);
        }

        self.users_private.insert(username.clone(), private_hash);
        self.users_public
            .insert(username.clone(), public_hash.clone());
        self.public_hash_to_username.insert(public_hash, username);

        Ok(())
    }

    pub fn delete_user(&mut self, username: &str) -> Result<(), AppError> {
        self.users_public
            .remove(username)
            .and_then(|public_hash| self.public_hash_to_username.remove(&public_hash))
            .and_then(|_| self.users_private.remove(username))
            .and(None)
            .ok_or(AppError::ConfigurationError(
                "User not found in users_public config.".to_string(),
            ))
    }

    pub fn list_users(&self) -> Vec<String> {
        self.users_public.keys().cloned().collect()
    }

    pub fn is_empty(&self) -> bool {
        self.users_public.is_empty()
    }

    pub fn contains_user(&self, username: &str) -> bool {
        self.users_public.contains_key(username)
    }

    pub fn verify_password(&self, username: &str, password: &str) -> Result<bool, AppError> {
        if let Some(private_hash) = self.users_private.get(username) {
            verify(password, private_hash)?;
            Ok(true)
        } else {
            Err(AppError::ConfigurationError(
                "User not found in users_private config.".to_string(),
            ))
        }
    }

    pub fn get_public_hash(&self, username: &str) -> Option<&String> {
        self.users_public.get(username)
    }

    pub fn get_username_from_public_hash(&self, public_hash: &str) -> Option<&String> {
        self.public_hash_to_username.get(public_hash)
    }
}
