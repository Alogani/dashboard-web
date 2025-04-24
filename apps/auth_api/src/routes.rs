use axum::{
    Router,
    extract::State,
    routing::{get, post},
};
use limiters_middleware::RateLimiter;
use state::AppState;
use tower_cookies::CookieManagerLayer;

mod check;
mod login;
mod login_page;
mod logout;

pub fn auth_routes(State(app_state): State<AppState>) -> Router<AppState> {
    let rate_limiter = RateLimiter::new(Some(15_000), Some(5));
    Router::new()
        .route("/login", get(login_page::login_page))
        .route("/login", post(login::login))
        .route("/check", get(check::check))
        .route("/logout", get(logout::logout))
        .with_state((app_state, rate_limiter))
        .layer(CookieManagerLayer::new())
}
