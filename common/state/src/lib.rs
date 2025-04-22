use std::{ops::Deref, sync::Arc};

use config::AppConfig;
use rate_limiter::RateLimiter;

#[derive(Clone)]
pub struct AppState {
    app_config: Arc<AppConfig>,
    rate_limiter: RateLimiter,
}

impl AppState {
    pub fn new(rate_limiter: RateLimiter, app_config: AppConfig) -> Self {
        AppState {
            app_config: Arc::new(app_config),
            rate_limiter,
        }
    }

    pub fn get_rate_limiter(&self) -> &RateLimiter {
        &self.rate_limiter
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
