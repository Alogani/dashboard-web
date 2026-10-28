use auth::remove_auth_cookie;
use axum::{
    extract::State,
    response::{Redirect, Response},
};

use limiters_middleware::RateLimiter;
use state::AppState;
use tower_cookies::Cookies;
use utils::with_nocache;

pub async fn logout(
    State((state, _rate_limiter)): State<(AppState, RateLimiter)>,
    cookies: Cookies,
) -> Response {
    remove_auth_cookie(&cookies, &state);

    with_nocache!(Redirect::to("/auth/login"))
}
