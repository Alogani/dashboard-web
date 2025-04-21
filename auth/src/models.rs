use common::{
    config::{AppConfig, UsersConfig},
    RateLimiter,
};
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct AuthState {
    pub rate_limiter: RateLimiter,
    pub app_config: Arc<RwLock<AppConfig>>,
    pub users_config: Arc<RwLock<UsersConfig>>,
}

#[derive(Deserialize)]
pub struct LoginForm {
    pub username: String,
    pub password: String,
}
