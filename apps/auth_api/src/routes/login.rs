use app_errors::AppError;
use askama::Template;
use axum::{
    extract::{Form, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect, Response},
};
use bcrypt::verify;
use config::UsersConfig;
use state::AppState;
use time::{Duration, OffsetDateTime};
use tower_cookies::{Cookie, Cookies};

use crate::models::LoginForm;
use crate::templates::LoginTemplate;

pub async fn login_page(
    cookies: Cookies,
    State(state): State<AppState>,
) -> Result<Response, AppError> {
    let mut welcome_message = String::new();
    let users_config = UsersConfig::from_file(state.get_app_config().get_users_file())?;

    // Check if user is already logged in
    if let Some(auth_cookie) = cookies.get("AuthUser") {
        if let Some(username) = users_config.get_username_from_public_hash(auth_cookie.value()) {
            welcome_message = format!("Welcome back, {}!", username);
        }
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

pub async fn login(
    State(state): State<AppState>,
    cookies: Cookies,
    Form(form): Form<LoginForm>,
) -> Result<Response, AppError> {
    tracing::debug!("Login attempt for user: {}", form.username);

    // Check if the user exists and the password is correct
    let users_config = UsersConfig::from_file(state.get_users_file())?;

    if let Some(password_hash) = users_config.get_username_from_public_hash(&form.username) {
        match verify(&form.password, password_hash) {
            Ok(true) => {
                tracing::debug!("Login successful for user: {}", form.username);

                // Generate auth hash for cookie
                let cookie_hash = users_config
                    .get_username_from_public_hash(&form.username)
                    .unwrap();

                // Set a cookie that expires in 24 hours
                let expiry = OffsetDateTime::now_utc() + Duration::hours(24);
                let domain = state.get_cookie_domain().to_string();
                let cookie = Cookie::build(("AuthUser", cookie_hash.clone()))
                    .path("/")
                    .domain(domain)
                    .expires(expiry)
                    .http_only(true);

                cookies.add(cookie.into());

                // Debug the cookie
                if let Some(cookie) = cookies.get("AuthUser") {
                    tracing::debug!("Cookie set: {}", cookie.value());
                } else {
                    tracing::warn!("Failed to set cookie!");
                }

                Ok(Redirect::to("/").into_response())
            }
            _ => {
                tracing::debug!("Invalid password for user: {}", form.username);

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
    } else {
        tracing::debug!("User not found: {}", form.username);

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
