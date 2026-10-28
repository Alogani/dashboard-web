use state::AppState;
use time::{Duration, OffsetDateTime};
use tower_cookies::{Cookie, Cookies, cookie::CookieBuilder, cookie::SameSite};

pub const COOKIE_NAME: &str = "AuthUser";

/// Returns the user owning the session referenced by the authentication cookie.
/// `Err` means a cookie was provided but doesn't match any valid session.
pub async fn identify_user_with_cookie(
    cookies: &Cookies,
    state: &AppState,
) -> Result<Option<String>, ()> {
    let cookie = if let Some(cookie) = cookies.get(COOKIE_NAME) {
        cookie
    } else {
        return Ok(None);
    };

    let token = cookie.value();
    if token.is_empty() {
        tracing::trace!("Empty cookie value for authentication");
        return Ok(None);
    };

    let username = state.get_sessions().get_username(token).ok_or(())?;
    if state.get_users_config().await.contains_user(&username) {
        Ok(Some(username))
    } else {
        Err(())
    }
}

pub fn set_auth_cookie(cookies: &Cookies, state: &AppState, username: &str) {
    tracing::trace!("Setting authentication cookie for user: {}", username);

    let duration = Duration::hours(state.get_cookie_duration() as i64);
    let token = state
        .get_sessions()
        .create(username, duration.unsigned_abs());
    let expiry = OffsetDateTime::now_utc() + duration;
    cookies.add(build_cookie(token, state).expires(expiry).into());
}

pub fn remove_auth_cookie(cookies: &Cookies, state: &AppState) {
    if let Some(cookie) = cookies.get(COOKIE_NAME) {
        state.get_sessions().revoke(cookie.value());
    }
    // Set in past to be sure the cookie is correctly removed
    let expiry = OffsetDateTime::now_utc() - Duration::hours(24);
    cookies.remove(build_cookie("".to_string(), state).expires(expiry).into());
    tracing::trace!("Cleared authentication cookie");
}

fn build_cookie<'a>(value: String, state: &AppState) -> CookieBuilder<'a> {
    let cookie = Cookie::build((COOKIE_NAME, value))
        .path("/")
        .secure(state.use_secure_cookies())
        .http_only(true)
        .same_site(SameSite::Lax);

    let domain = state.get_cookie_domain().to_string();
    if !domain.is_empty() {
        cookie.domain(domain)
    } else {
        cookie
    }
}
