use app_errors::AppError;
use askama::Template;
use axum::{
    extract::{Form, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect, Response},
};
use state::AppState;
use tower_cookies::Cookies;

use crate::{auth_cookie::set_auth_cookie, templates::LoginTemplate};

use serde::Deserialize;

#[derive(Deserialize)]
pub struct LoginForm {
    pub username: String,
    pub password: String,
}

pub async fn login(
    State(state): State<AppState>,
    cookies: Cookies,
    Form(form): Form<LoginForm>,
) -> Result<Response, AppError> {
    tracing::debug!("Login attempt for user: {}", form.username);

    // Check if the user exists and the password is correct
    let users_config = state.get_users_config().await;

    if let Ok(true) = users_config.verify_password(&form.username, &form.password) {
        tracing::debug!("Login successful for user: {}", form.username);

        set_auth_cookie(&cookies, &state, &form.username).await?;

        Ok(Redirect::to("/").into_response())
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
