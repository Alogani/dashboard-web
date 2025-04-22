use axum::{
    Router,
    extract::State,
    routing::{get, post},
};
use state::AppState;
use tower_cookies::CookieManagerLayer;

mod check_auth;
mod login;
mod logout;

pub use check_auth::start_cache_cleanup;

pub fn auth_routes(State(state): State<AppState>) -> Router<AppState> {
    Router::new()
        .route("/login", get(login::login_page))
        .route("/login", post(login::login))
        .route("/check_auth", get(check_auth::check_cookie))
        .route("/logout", get(logout::logout))
        .with_state(state)
        .layer(CookieManagerLayer::new())
}
