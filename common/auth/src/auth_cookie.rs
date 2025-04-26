use app_errors::AppError;
use state::AppState;
use time::{Duration, OffsetDateTime};
use tower_cookies::{Cookie, Cookies};

const COOKIE_NAME: &str = "AuthUser";

pub async fn identify_user_with_cookie(
    cookies: &Cookies,
    state: &AppState,
) -> Result<Option<String>, ()> {
    let cookie = if let Some(cookie) = cookies.get(COOKIE_NAME) {
        cookie
    } else {
        return Ok(None);
    };

    let public_hash = cookie.value().to_string();
    if public_hash.is_empty() {
        tracing::trace!("Empty cookie value for authentication");
        return Ok(None);
    };

    state
        .get_users_config()
        .await
        .get_username_from_public_hash(&public_hash)
        .map(|username| Some(username.clone()))
        .ok_or(())
}

pub async fn set_auth_cookie(
    cookies: &Cookies,
    state: &AppState,
    username: &str,
) -> Result<(), AppError> {
    let users_config = state.get_users_config().await;
    let public_hash =
        users_config
            .get_public_hash(username)
            .cloned()
            .ok_or(AppError::ConfigurationError(
                "No username found.".to_string(),
            ))?;

    let expiry = OffsetDateTime::now_utc() + Duration::hours(state.get_cookie_duration() as i64);

    let cookie = Cookie::build((COOKIE_NAME, public_hash))
        .path("/")
        .expires(expiry)
        .secure(state.use_secure_cookies())
        .http_only(true);
    let domain = state.get_cookie_domain().to_string();
    let cookie = if !domain.is_empty() {
        cookie.domain(domain)
    } else {
        cookie
    };
    tracing::trace!("Setting authentication cookie for user: {}", username);

    cookies.add(cookie.into());

    Ok(())
}

pub fn clear_auth_cookie(cookies: &Cookies, state: &AppState) {
    let mut cookie = Cookie::new(COOKIE_NAME, "");
    let domain = state.get_cookie_domain().to_string();
    if !domain.is_empty() {
        cookie.set_domain(domain);
    }

    // Set other attributes to match how the cookie was set
    cookie.set_http_only(true);
    cookie.set_secure(state.use_secure_cookies());

    cookies.remove(cookie);
    tracing::trace!("Cleared authentication cookie");
}
