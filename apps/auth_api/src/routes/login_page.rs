use app_errors::AppError;
use askama::Template;
use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};

use limiters_middleware::RateLimiter;
use state::AppState;

use tower_cookies::Cookies;

use crate::templates::LoginTemplate;
use auth::identify_user_with_cookie;
use http::header::{CACHE_CONTROL, VARY};
use http::HeaderValue;

pub async fn login_page(
    cookies: Cookies,
    State((state, _)): State<(AppState, RateLimiter)>,
) -> Result<Response, AppError> {
    let mut welcome_message = String::new();

    match identify_user_with_cookie(&cookies, &state).await {
        Ok(Some(username)) => {
            welcome_message = format!("Welcome back, {}!", username);
        }
        Ok(None) => {}
        Err(_) => {
            welcome_message = format!("Unknown user, please login");
        }
    }

    let template = LoginTemplate {
        error_message: "",
        welcome_message,
    };

    match template.render() {
        Ok(html) => {
            let mut res = Html(html).into_response();
            res.headers_mut().insert(
                CACHE_CONTROL,
                HeaderValue::from_static("no-store, no-cache, must-revalidate"),
            );
            res.headers_mut().insert(VARY, HeaderValue::from_static("Cookie"));
            Ok(res)
        }
        Err(err) => {
            tracing::error!("Template error: {}", err);
            Ok(StatusCode::INTERNAL_SERVER_ERROR.into_response())
        }
    }
}
