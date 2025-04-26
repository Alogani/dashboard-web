use axum::{
    extract::State,
    response::{IntoResponse, Redirect, Response},
};

use limiters_middleware::RateLimiter;
use state::AppState;
use tower_cookies::Cookies;

use auth::clear_auth_cookie;

pub async fn logout(
    State((state, _rate_limiter)): State<(AppState, RateLimiter)>,
    cookies: Cookies,
) -> Response {
    clear_auth_cookie(&cookies, &state);

    Redirect::to("/auth/login").into_response()
}
