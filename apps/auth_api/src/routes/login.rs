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

use crate::templates::{LoginError, LoginTemplate};
use auth::redirect_cookie::consume_redirect_cookie;
use auth::set_auth_cookie;
use http::HeaderValue;
use http::header::{CACHE_CONTROL, PRAGMA, VARY};

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
            Ok(html) => Some(Html(html).into_response()),
            Err(err) => {
                tracing::error!("Template error: {}", err);
                Some(StatusCode::INTERNAL_SERVER_ERROR.into_response())
            }
        };
    }
    None
}

/// Extract the real client IP from headers when behind a reverse proxy
fn get_real_ip(headers: &HeaderMap, socket_addr: &SocketAddr) -> String {
    // Try X-Forwarded-For first
    if let Some(forwarded_for) = headers.get("X-Forwarded-For") {
        if let Ok(forwarded_str) = forwarded_for.to_str() {
            // X-Forwarded-For can contain multiple IPs, take the first one
            if let Some(client_ip) = forwarded_str.split(',').next() {
                return client_ip.trim().to_string();
            }
        }
    }

    // Try X-Real-IP as fallback
    if let Some(real_ip) = headers.get("X-Real-IP") {
        if let Ok(ip_str) = real_ip.to_str() {
            return ip_str.to_string();
        }
    }

    // Fallback to socket address if headers aren't available
    socket_addr.ip().to_string()
}

pub async fn login(
    State((state, rate_limiter)): State<(AppState, RateLimiter)>,
    cookies: Cookies,
    headers: HeaderMap,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Form(form): Form<LoginForm>,
) -> Result<Response, AppError> {
    tracing::trace!("Login attempt for user: {}", form.username);

    // Get the real client IP
    let ip = get_real_ip(&headers, &addr);

    // Rate limit the login attempts
    if let Some(response) = check_rate_limit(&rate_limiter, &ip) {
        return Ok(response);
    }

    // Check if the user exists and the password is correct
    let users_config = state.get_users_config().await;

    if let Ok(true) = users_config.verify_password(&form.username, &form.password) {
        tracing::debug!("Login successful for user: {}", form.username);
        rate_limiter.clear(&ip);

        set_auth_cookie(&cookies, &state, &form.username).await?;

        // Get the redirect URL or path
        let (subdomain, route) =
            consume_redirect_cookie(&cookies, &state).unwrap_or_else(|| (None, "/".to_string()));

        if state.is_access_allowed(
            subdomain.as_ref().map(|s| s.as_str()).clone(),
            &route,
            Some(&form.username),
        ) {
            tracing::info!(
                "Redirect after login: subdomain: {:?}, path: {}",
                subdomain,
                route
            );

            // Have to reconstruct the full URL
            let cookie_domain = state.get_cookie_domain();
            let redirect_url = if let Some(subdomain) = subdomain {
                format!("https://{}.{}{}", subdomain, cookie_domain, route)
            } else {
                route
            };

            let mut res = Redirect::to(&redirect_url).into_response();
            res.headers_mut().insert(
                CACHE_CONTROL,
                HeaderValue::from_static("no-store, no-cache, must-revalidate"),
            );
            res.headers_mut()
                .insert(PRAGMA, HeaderValue::from_static("no-cache"));
            res.headers_mut()
                .insert(VARY, HeaderValue::from_static("Cookie"));
            return Ok(res);
        } else {
            tracing::warn!(
                "User {} is not allowed to access route: {} at subdomain: {:?}",
                form.username,
                route,
                subdomain
            );
            let template = LoginTemplate {
                error_message: "You don't have permission to access the requested subdomain",
                welcome_message: String::new(),
            };
            return match template.render() {
                Ok(html) => {
                    let mut res = (StatusCode::FORBIDDEN, Html(html)).into_response();
                    res.headers_mut().insert(
                        CACHE_CONTROL,
                        HeaderValue::from_static("no-store, no-cache, must-revalidate"),
                    );
                    res.headers_mut()
                        .insert(VARY, HeaderValue::from_static("Cookie"));
                    Ok(res)
                }
                Err(err) => {
                    tracing::error!("Template error: {}", err);
                    Ok(StatusCode::INTERNAL_SERVER_ERROR.into_response())
                }
            };
        }
    } else {
        tracing::warn!(
            "Authentication failed: Invalid username or password for user '{}' from IP {}",
            form.username,
            ip
        );

        let template = LoginTemplate {
            error_message: "Invalid username or password",
            welcome_message: String::new(),
        };

        match template.render() {
            Ok(html) => {
                let mut res = (StatusCode::UNAUTHORIZED, Html(html)).into_response();
                res.headers_mut().insert(
                    CACHE_CONTROL,
                    HeaderValue::from_static("no-store, no-cache, must-revalidate"),
                );
                res.headers_mut()
                    .insert(VARY, HeaderValue::from_static("Cookie"));
                Ok(res)
            }
            Err(err) => {
                tracing::error!("Template error: {}", err);
                Ok(StatusCode::INTERNAL_SERVER_ERROR.into_response())
            }
        }
    }
}
