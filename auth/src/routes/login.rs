use askama::Template;
use axum::{
    extract::{Form, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect, Response},
};
use bcrypt::verify;
use time::{Duration, OffsetDateTime};
use tower_cookies::{Cookie, Cookies};

use crate::{
    models::{AuthState, LoginForm},
    templates::LoginTemplate,
};

pub async fn login_page(cookies: Cookies, State(state): State<AuthState>) -> impl IntoResponse {
    let mut welcome_message = String::new();

    // Check if user is already logged in
    if let Some(auth_cookie) = cookies.get("AuthUser") {
        let users_config = state.users_config.read().await;
        if let Some(username) = users_config.get_username_from_hash(auth_cookie.value()) {
            welcome_message = format!("Welcome back, {}!", username);
        }
    }

    let template = LoginTemplate {
        error_message: "",
        welcome_message,
    };

    match template.render() {
        Ok(html) => Html(html).into_response(),
        Err(err) => {
            tracing::error!("Template error: {}", err);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn login(
    State(state): State<AuthState>,
    cookies: Cookies,
    Form(form): Form<LoginForm>,
) -> Response {
    tracing::debug!("Login attempt for user: {}", form.username);

    // Check if the user exists and the password is correct
    let users_config = state.users_config.read().await;

    if let Some(password_hash) = users_config.get_password_hash(&form.username) {
        match verify(&form.password, password_hash) {
            Ok(true) => {
                tracing::debug!("Login successful for user: {}", form.username);

                // Generate auth hash for cookie
                let cookie_hash = users_config.get_user_hash(&form.username).unwrap();

                // Set a cookie that expires in 24 hours
                let expiry = OffsetDateTime::now_utc() + Duration::hours(24);
                let cookie = Cookie::build(("AuthUser", cookie_hash))
                    .path("/")
                    .domain(state.cookie_domain)
                    .expires(expiry)
                    .http_only(true);

                cookies.add(cookie.into());

                // Debug the cookie
                if let Some(cookie) = cookies.get("AuthUser") {
                    tracing::debug!("Cookie set: {}", cookie.value());
                } else {
                    tracing::warn!("Failed to set cookie!");
                }

                Redirect::to("/").into_response()
            }
            _ => {
                tracing::debug!("Invalid password for user: {}", form.username);

                let template = LoginTemplate {
                    error_message: "Invalid username or password",
                    welcome_message: String::new(),
                };

                match template.render() {
                    Ok(html) => (StatusCode::UNAUTHORIZED, Html(html)).into_response(),
                    Err(err) => {
                        tracing::error!("Template error: {}", err);
                        StatusCode::INTERNAL_SERVER_ERROR.into_response()
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
            Ok(html) => (StatusCode::UNAUTHORIZED, Html(html)).into_response(),
            Err(err) => {
                tracing::error!("Template error: {}", err);
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}
