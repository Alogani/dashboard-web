use std::net::SocketAddr;

use app_errors::AppError;
use askama::Template;
use axum::{
    extract::{ConnectInfo, Form, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect, Response},
};
use rate_limiter::RateLimiter;
use state::AppState;
use tower_cookies::Cookies;
use url::Url;

use crate::templates::{LoginError, LoginTemplate};
use auth::{get_redirect_cookie, set_auth_cookie};

use serde::Deserialize;

#[derive(Deserialize)]
pub struct LoginForm {
    pub username: String,
    pub password: String,
}

/// Checks if the request should be rate limited based on IP address
fn check_rate_limit(rate_limiter: &RateLimiter<u64>, ip: &str) -> Option<Response> {
    if !rate_limiter.check_rate_limit(ip, |rate_ok, attempt_count| {
        tracing::trace!(
            "Rate limit check: rate_ok: {}, attempt_count: {}",
            rate_ok,
            attempt_count
        );
        (rate_ok || attempt_count < 3, attempt_count + 1)
    }) {
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
    // In case of success, clear the rate limiter for the IP address
    rate_limiter.clear(ip);
    None
}

pub async fn login(
    State((state, rate_limiter)): State<(AppState, RateLimiter<u64>)>,
    cookies: Cookies,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Form(form): Form<LoginForm>,
) -> Result<Response, AppError> {
    tracing::debug!("Login attempt for user: {}", form.username);

    // Rate limit the login attempts
    let ip = addr.ip().to_string();
    if let Some(response) = check_rate_limit(&rate_limiter, &ip) {
        return Ok(response);
    }

    // Check if the user exists and the password is correct
    let users_config = state.get_users_config().await;

    if let Ok(true) = users_config.verify_password(&form.username, &form.password) {
        tracing::debug!("Login successful for user: {}", form.username);

        set_auth_cookie(&cookies, &state, &form.username).await?;

        // Get the redirect URL or path
        let redirect_to = get_redirect_cookie(&cookies).unwrap_or_else(|| "/".to_string());
        tracing::debug!("Redirect after login: {}", redirect_to);

        // Check if it's a full URL or just a path
        let is_full_url = redirect_to.starts_with("http://") || redirect_to.starts_with("https://");

        if is_full_url {
            // For full URLs, we need to check if the user has access to the subdomain
            match Url::parse(&redirect_to) {
                Ok(url) => {
                    let host = url.host_str().unwrap_or("");
                    // Extract subdomain from host (e.g., "incus" from "incus.nginx.lan")
                    let subdomain = host.split('.').next().unwrap_or("");

                    if !subdomain.is_empty()
                        && !state.is_subdomain_allowed(subdomain, Some(&form.username))
                    {
                        tracing::warn!(
                            "User {} is not allowed to access subdomain {}",
                            form.username,
                            subdomain
                        );

                        let template = LoginTemplate {
                            error_message: "You don't have permission to access the requested subdomain",
                            welcome_message: String::new(),
                        };

                        return match template.render() {
                            Ok(html) => Ok((StatusCode::FORBIDDEN, Html(html)).into_response()),
                            Err(err) => {
                                tracing::error!("Template error: {}", err);
                                Ok(StatusCode::INTERNAL_SERVER_ERROR.into_response())
                            }
                        };
                    }

                    // User is allowed to access the subdomain, redirect to the full URL
                    return Ok(Redirect::to(&redirect_to).into_response());
                }
                Err(_) => {
                    // If URL parsing fails, fall back to path-based check
                    tracing::warn!("Failed to parse redirect URL: {}", redirect_to);
                }
            }
        }

        // For paths or if URL parsing failed, check if the user is allowed to access the path
        let path = if is_full_url {
            extract_path_from_url(&redirect_to)
        } else {
            redirect_to.clone()
        };

        let app_config = state.get_app_config();
        if app_config.is_route_allowed(&path, Some(&form.username)) {
            Ok(Redirect::to(&redirect_to).into_response())
        } else {
            tracing::warn!(
                "User {} is not allowed to access path {}",
                form.username,
                path
            );

            let template = LoginTemplate {
                error_message: "You don't have permission to access the requested page",
                welcome_message: String::new(),
            };

            match template.render() {
                Ok(html) => Ok((StatusCode::FORBIDDEN, Html(html)).into_response()),
                Err(err) => {
                    tracing::error!("Template error: {}", err);
                    Ok(StatusCode::INTERNAL_SERVER_ERROR.into_response())
                }
            }
        }
    } else {
        tracing::debug!("Invalid username or password for {}", form.username);

        let template = LoginTemplate {
            error_message: "Invalid username or password",
            welcome_message: String::new(),
        };

        match template.render() {
            Ok(html) => Ok((StatusCode::UNAUTHORIZED, Html(html)).into_response()),
            Err(err) => {
                tracing::error!("Template error: {}", err);
                Ok(StatusCode::INTERNAL_SERVER_ERROR.into_response())
            }
        }
    }
}

fn extract_path_from_url(url_str: &str) -> String {
    match Url::parse(url_str) {
        Ok(url) => {
            let path = url.path();
            // If path is empty, return "/" as the root path
            if path.is_empty() {
                "/".to_string()
            } else {
                // Include query parameters if present
                if let Some(query) = url.query() {
                    format!("{}?{}", path, query)
                } else {
                    path.to_string()
                }
            }
        }
        Err(_) => {
            tracing::warn!("Failed to parse URL in extract_path_from_url: {}", url_str);
            url_str.to_string()
        }
    }
}
