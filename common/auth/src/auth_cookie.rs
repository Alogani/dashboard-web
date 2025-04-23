use app_errors::AppError;
use state::AppState;
use time::{Duration, OffsetDateTime};
use tower_cookies::{Cookie, Cookies};

const COOKIE_NAME: &str = "auth_user";
const COOKIE_DURATION: Duration = Duration::hours(24);

pub async fn identify_user_with_cookie(cookies: &Cookies, state: &AppState) -> Option<String> {
    let public_hash = cookies.get(COOKIE_NAME).map(|c| c.value().to_string())?;

    state
        .get_users_config()
        .await
        .get_username_from_public_hash(&public_hash)
        .cloned()
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

    let expiry = OffsetDateTime::now_utc() + COOKIE_DURATION;
    let domain = state.get_cookie_domain().to_string();
    let cookie = Cookie::build(("AuthUser", public_hash))
        .path("/")
        .domain(domain)
        .expires(expiry)
        .secure(true)
        .http_only(true);

    cookies.add(cookie.into());

    Ok(())
}

pub fn clear_auth_cookie(cookies: &Cookies) {
    let mut cookie = Cookie::new(COOKIE_NAME, "");
    cookie.set_path("/");
    cookies.remove(cookie);
}
