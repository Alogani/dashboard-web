use bcrypt::{DEFAULT_COST, hash, verify};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::sync::LazyLock;

use app_errors::AppError;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct UsersConfig {
    // Username alongside the bcrypt hash of their password
    #[serde(flatten)]
    users: HashMap<String, String>,
}

// Hash verified when the user doesn't exist, so that the response time doesn't reveal
// whether a username is valid
static DUMMY_HASH: LazyLock<String> =
    LazyLock::new(|| hash("dummy password", DEFAULT_COST).expect("bcrypt hashing failed"));

/// Verifies a password against a bcrypt hash. When no hash is given, a dummy one is
/// still verified to keep a constant response time, and `false` is returned.
///
/// This is CPU intensive (~0.25s), so it should not be called directly from async code.
pub fn verify_password_hash(password: &str, password_hash: Option<&str>) -> bool {
    match password_hash {
        Some(password_hash) => verify(password, password_hash).unwrap_or_else(|err| {
            tracing::error!("Failed to verify password hash: {}", err);
            false
        }),
        None => {
            let _ = verify(password, &DUMMY_HASH);
            false
        }
    }
}

impl UsersConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, AppError> {
        let content = fs::read_to_string(&path).map_err(|e| {
            AppError::ConfigurationError(format!(
                "Error encountered while trying to read users config at {:?}: {:?}",
                path.as_ref().display(),
                e
            ))
        })?;
        toml::from_str(&content).map_err(|e| {
            AppError::ConfigurationError(format!(
                "Error encountered while trying to read users config at {:?}: {:?}",
                path.as_ref().display(),
                e
            ))
        })
    }

    /// Writes the users file, readable only by its owner as it contains password hashes
    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), AppError> {
        let serialized = toml::to_string(&self)?;
        let mut options = fs::OpenOptions::new();
        options.write(true).create(true).truncate(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
            options.mode(0o600);
            // The mode is only applied on creation
            if path.as_ref().exists() {
                fs::set_permissions(&path, fs::Permissions::from_mode(0o600))?;
            }
        }
        let mut file = options.open(&path)?;
        file.write_all(serialized.as_bytes())?;
        Ok(())
    }

    pub fn add_or_update_user(&mut self, username: String, password: &str) -> Result<(), AppError> {
        let password_hash = hash(password, DEFAULT_COST)?;
        self.users.insert(username, password_hash);
        Ok(())
    }

    pub fn delete_user(&mut self, username: &str) -> Result<(), AppError> {
        self.users
            .remove(username)
            .map(|_| ())
            .ok_or(AppError::ConfigurationError(
                "User not found in users config.".to_string(),
            ))
    }

    pub fn list_users(&self) -> Vec<String> {
        self.users.keys().cloned().collect()
    }

    pub fn is_empty(&self) -> bool {
        self.users.is_empty()
    }

    pub fn contains_user(&self, username: &str) -> bool {
        self.users.contains_key(username)
    }

    pub fn get_password_hash(&self, username: &str) -> Option<&String> {
        self.users.get(username)
    }

    pub fn verify_password(&self, username: &str, password: &str) -> bool {
        verify_password_hash(
            password,
            self.get_password_hash(username).map(String::as_str),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_password() {
        let mut config = UsersConfig::new();
        config
            .add_or_update_user("alice".to_string(), "correct horse")
            .unwrap();

        assert!(config.verify_password("alice", "correct horse"));
        assert!(!config.verify_password("alice", "wrong password"));
        assert!(!config.verify_password("alice", ""));
        assert!(!config.verify_password("bob", "correct horse"));
    }

    #[test]
    fn test_delete_user() {
        let mut config = UsersConfig::new();
        config
            .add_or_update_user("alice".to_string(), "pwd")
            .unwrap();

        assert!(config.delete_user("alice").is_ok());
        assert!(!config.contains_user("alice"));
        assert!(config.delete_user("alice").is_err());
    }

    #[test]
    fn test_users_file_roundtrip() {
        let path = std::env::temp_dir().join(format!("users_db_test_{}.txt", std::process::id()));
        let mut config = UsersConfig::new();
        config
            .add_or_update_user("alice".to_string(), "pwd")
            .unwrap();
        config.to_file(&path).unwrap();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = fs::metadata(&path).unwrap().permissions().mode();
            assert_eq!(mode & 0o777, 0o600);
        }

        let loaded = UsersConfig::from_file(&path).unwrap();
        fs::remove_file(&path).unwrap();
        assert!(loaded.verify_password("alice", "pwd"));
    }
}
