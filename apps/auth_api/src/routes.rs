use axum::{
    Router,
    extract::State,
    routing::{get, post},
};
use state::AppState;
use tower_cookies::CookieManagerLayer;

mod check_auth;
mod login;
mod login_page;
mod logout;

pub fn auth_routes(State(state): State<AppState>) -> Router<AppState> {
    Router::new()
        .route("/login", get(login_page::login_page))
        .route("/login", post(login::login))
        .route("/check_auth", get(check_auth::check_auth))
        .route("/logout", get(logout::logout))
        .with_state(state)
        .layer(CookieManagerLayer::new())
}
