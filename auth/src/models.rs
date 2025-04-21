use common::{
    config::{RoutesConfig, UsersConfig},
    RateLimiter,
};
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct AuthState {
    pub rate_limiter: RateLimiter,
    pub cookie_domain: String,
    pub router_address: String,
    pub users_config: Arc<RwLock<UsersConfig>>,
    pub routes_config: Arc<RwLock<RoutesConfig>>,
}

#[derive(Deserialize)]
pub struct LoginForm {
    pub username: String,
    pub password: String,
}
