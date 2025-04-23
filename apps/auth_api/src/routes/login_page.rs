use app_errors::AppError;
use askama::Template;
use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};

use rate_limiter::RateLimiter;
use state::AppState;

use tower_cookies::Cookies;

use crate::templates::LoginTemplate;
use auth::identify_user_with_cookie;

pub async fn login_page(
    cookies: Cookies,
    State((state, _)): State<(AppState, RateLimiter<u64>)>,
) -> Result<Response, AppError> {
    let mut welcome_message = String::new();

    if let Some(username) = identify_user_with_cookie(&cookies, &state).await {
        welcome_message = format!("Welcome back, {}!", username);
    }

    let template = LoginTemplate {
        error_message: "",
        welcome_message,
    };

    match template.render() {
        Ok(html) => Ok(Html(html).into_response()),
        Err(err) => {
            tracing::error!("Template error: {}", err);
            Ok(StatusCode::INTERNAL_SERVER_ERROR.into_response())
        }
    }
}
