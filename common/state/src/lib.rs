mod logging;
mod sessions;
mod signal_handlers;

use app_errors::AppError;
use logging::{TracingReloadHandler, reload_logging, setup_logging};
pub use sessions::SessionStore;
use signal_handlers::spawn_sighup_watcher;
use std::{ops::Deref, sync::Arc};
use tokio::sync::{RwLock, RwLockReadGuard};

use config::{AppConfig, UsersConfig};

#[derive(Clone)]
pub struct AppState {
    app_config: Arc<AppConfig>,
    user_config: Arc<RwLock<UsersConfig>>,
    sessions: Arc<SessionStore>,
    tracing_reloader: Arc<Option<TracingReloadHandler>>,
}

impl AppState {
    pub fn init(app_config: AppConfig) -> Self {
        let tracing_reloader =
            setup_logging(&app_config.get_log_level(), &app_config.get_log_file());

        let users_config = match UsersConfig::from_file(app_config.get_usersdb_path()) {
            Ok(config) => config,
            Err(err) => {
                tracing::warn!(
                    "Failed to load users configuration: {}. Using empty configuration. \
                    It is recommended to use the command line CLI to create and manage users_db file.",
                    err
                );
                UsersConfig::new()
            }
        };
        let result = AppState {
            app_config: Arc::new(app_config),
            user_config: Arc::new(RwLock::new(users_config)),
            sessions: Arc::new(SessionStore::default()),
            tracing_reloader: Arc::new(tracing_reloader),
        };
        spawn_sighup_watcher(result.clone());
        result
    }

    pub async fn reload_all_config(&self) -> Result<(), AppError> {
        self.reload_user_config().await?;
        reload_logging(self.tracing_reloader.clone(), self.get_log_file());
        Ok(())
    }

    /// Reloads the users file and revokes the sessions of the users that were
    /// deleted or whose password changed
    pub async fn reload_user_config(&self) -> Result<(), AppError> {
        let new_config = UsersConfig::from_file(self.get_usersdb_path())?;
        let mut user_config = self.user_config.write().await;
        self.sessions.revoke_users(|username| {
            user_config.get_password_hash(username) != new_config.get_password_hash(username)
        });
        *user_config = new_config;
        Ok(())
    }

    pub async fn get_users_config(&self) -> RwLockReadGuard<'_, UsersConfig> {
        self.user_config.read().await
    }

    pub fn get_sessions(&self) -> &SessionStore {
        &self.sessions
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
