use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_cookies::CookieManagerLayer;

use crate::models::AuthState;
use common::config::{RoutesConfig, UsersConfig};

mod check_cookie;
mod login;
mod logout;

pub fn auth_routes(
    cookie_domain: String,
    router_address: String,
    users_config: Arc<RwLock<UsersConfig>>,
    routes_config: Arc<RwLock<RoutesConfig>>,
) -> Router<AuthState> {
    let rate_limiter = common::RateLimiter::new(None);
    let state = AuthState {
        rate_limiter,
        cookie_domain,
        router_address,
        users_config,
        routes_config,
    };

    Router::new()
        .route("/login", get(login::login_page))
        .route("/login", post(login::login))
        .route("/check_cookie", get(check_cookie::check_cookie))
        .route("/logout", get(logout::logout))
        .with_state(state)
        .layer(CookieManagerLayer::new())
}
