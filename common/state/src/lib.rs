use app_errors::AppError;
use std::{ops::Deref, sync::Arc};
use tokio::sync::{RwLock, RwLockReadGuard};

use config::{AppConfig, UsersConfig};

#[derive(Clone)]
pub struct AppState {
    app_config: Arc<AppConfig>,
    user_config: Arc<RwLock<UsersConfig>>,
}

impl AppState {
    pub fn new(app_config: AppConfig) -> Self {
        let users_config = UsersConfig::from_file(app_config.get_users_file()).unwrap();
        AppState {
            app_config: Arc::new(app_config),
            user_config: Arc::new(RwLock::new(users_config)),
        }
    }

    pub async fn reload_user_config(&self) -> Result<(), AppError> {
        let mut user_config = self.user_config.write().await;
        *user_config = UsersConfig::from_file(self.get_users_file())?;
        Ok(())
    }

    pub async fn get_users_config(&self) -> RwLockReadGuard<UsersConfig> {
        self.user_config.read().await
    }

    pub fn get_app_config(&self) -> &AppConfig {
        &self.app_config
    }
}

impl Deref for AppState {
    type Target = AppConfig;

    fn deref(&self) -> &AppConfig {
        self.get_app_config()
    }
}
