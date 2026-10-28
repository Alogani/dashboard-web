use std::net::SocketAddr;

use app_errors::AppError;
use askama::Template;
use axum::{
    extract::{ConnectInfo, Form, State},
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Redirect, Response},
};
use limiters_middleware::RateLimiter;
use state::AppState;
use tower_cookies::Cookies;
use utils::http_helpers::get_client_ip;
use utils::with_nocache;

use crate::templates::{LoginError, LoginTemplate};
use auth::redirect_cookie::consume_redirect_cookie;
use auth::set_auth_cookie;

use serde::Deserialize;

#[derive(Deserialize)]
pub struct LoginForm {
    pub username: String,
    pub password: String,
}

/// Checks if the request should be rate limited based on IP address
fn check_rate_limit(rate_limiter: &RateLimiter, ip: &str) -> Option<Response> {
    if !rate_limiter.check_limit(ip) {
        let template = LoginError {
            message: "Too many logging attemps, please wait.",
        };
        return match template.render() {
            Ok(html) => Some(with_nocache!((StatusCode::TOO_MANY_REQUESTS, Html(html)))),
            Err(err) => {
                tracing::error!("Template error: {}", err);
                Some(StatusCode::INTERNAL_SERVER_ERROR.into_response())
            }
        };
    }
    None
}

pub async fn login(
    State((state, rate_limiter)): State<(AppState, RateLimiter)>,
    cookies: Cookies,
    headers: HeaderMap,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Form(form): Form<LoginForm>,
) -> Result<Response, AppError> {
    // Usernames are user controlled: always log them with Debug formatting (quoted and escaped)
    // to prevent log injection, which could be used to get any IP banned by fail2ban
    tracing::trace!("Login attempt for user: {:?}", form.username);

    // Get the real client IP
    let ip = get_client_ip(&headers, &addr);

    // Rate limit the login attempts
    if let Some(response) = check_rate_limit(&rate_limiter, &ip) {
        return Ok(response);
    }

    // Check if the user exists and the password is correct.
    // Bcrypt is CPU intensive, so it is run outside of the async executor
    let password_hash = state
        .get_users_config()
        .await
        .get_password_hash(&form.username)
        .cloned();
    let password = form.password;
    let is_valid = tokio::task::spawn_blocking(move || {
        config::verify_password_hash(&password, password_hash.as_deref())
    })
    .await
    .map_err(|e| AppError::ServerError(format!("Password verification failed: {}", e)))?;

    if is_valid {
        tracing::debug!("Login successful for user: {:?}", form.username);
        rate_limiter.clear(&ip);

        set_auth_cookie(&cookies, &state, &form.username);

        // Get the redirect URL or path
        let (subdomain, route) =
            consume_redirect_cookie(&cookies, &state).unwrap_or_else(|| (None, "/".to_string()));

        if state.is_access_allowed(subdomain.as_deref(), &route, Some(&form.username)) {
            tracing::info!(
                "Redirect after login: subdomain: {:?}, path: {}",
                subdomain,
                route
            );

            // Have to reconstruct the full URL
            let cookie_domain = state.get_cookie_domain();
            let redirect_url = match subdomain {
                Some(subdomain) if !cookie_domain.is_empty() => {
                    format!("https://{}.{}{}", subdomain, cookie_domain, route)
                }
                _ => route,
            };

            return Ok(with_nocache!(Redirect::to(&redirect_url)));
        } else {
            tracing::warn!(
                "User {:?} is not allowed to access route: {} at subdomain: {:?}",
                form.username,
                route,
                subdomain
            );
            let template = LoginTemplate {
                error_message: "You don't have permission to access the requested subdomain",
                welcome_message: String::new(),
            };
            return match template.render() {
                Ok(html) => Ok(with_nocache!((StatusCode::FORBIDDEN, Html(html)))),
                Err(err) => {
                    tracing::error!("Template error: {}", err);
                    Ok(StatusCode::INTERNAL_SERVER_ERROR.into_response())
                }
            };
        }
    } else {
        // The IP is placed before the user controlled username so that it can't be spoofed
        // in the fail2ban regex
        tracing::warn!(
            "Authentication failed from IP {}: invalid username or password for user {:?}",
            ip,
            form.username
        );

        let template = LoginTemplate {
            error_message: "Invalid username or password",
            welcome_message: String::new(),
        };

        match template.render() {
            Ok(html) => Ok(with_nocache!((StatusCode::UNAUTHORIZED, Html(html)))),

            Err(err) => {
                tracing::error!("Template error: {}", err);
                Ok(StatusCode::INTERNAL_SERVER_ERROR.into_response())
            }
        }
    }
}
