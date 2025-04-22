use axum::{
    Router,
    routing::{get, post},
};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_cookies::CookieManagerLayer;

use crate::models::AuthState;
use config::AppConfig;

mod check_cookie;
mod login;
mod logout;

pub use check_cookie::start_cache_cleanup;

pub fn auth_routes(
    app_config: Arc<RwLock<AppConfig>>,
    users_config: Arc<RwLock<config::UsersConfig>>,
) -> Router<AuthState> {
    let rate_limiter = common::RateLimiter::new(None);
    let state = AuthState {
        rate_limiter,
        app_config,
        users_config,
    };

    Router::new()
        .route("/login", get(login::login_page))
        .route("/login", post(login::login))
        .route("/check_cookie", get(check_cookie::check_cookie))
        .route("/logout", get(logout::logout))
        .with_state(state)
        .layer(CookieManagerLayer::new())
}
