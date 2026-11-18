use http::{HeaderMap, header::COOKIE};
use state::AppState;
use time::{Duration, OffsetDateTime};
use tower_cookies::{Cookie, Cookies, cookie::CookieBuilder, cookie::SameSite};

pub const COOKIE_NAME: &str = "AuthUser";

/// Returns the user owning the session referenced by the authentication cookie.
/// `Err` means a cookie was provided but doesn't match any valid session.
pub async fn identify_user_with_cookie(
    headers: &HeaderMap,
    state: &AppState,
) -> Result<Option<String>, ()> {
    let mut cookies = auth_cookies(headers).peekable();
    if cookies.peek().is_none() {
        return Ok(None);
    }

    let users = state.get_users_config().await;
    cookies
        .filter_map(|cookie| state.get_sessions().get_username(cookie.value()))
        .filter(|username| users.contains_user(username))
        // Preserve the last-cookie precedence when multiple sessions are valid.
        .last()
        .map(Some)
        .ok_or(())
}

fn auth_cookies(headers: &HeaderMap) -> impl Iterator<Item = Cookie<'_>> {
    // Browsers can send the same cookie name from different scopes. The cookie jar
    // keeps only the last value for each name, which may refer to an expired session.
    headers
        .get_all(COOKIE)
        .iter()
        .filter_map(|value| std::str::from_utf8(value.as_bytes()).ok())
        .flat_map(Cookie::split_parse_encoded)
        .filter_map(Result::ok)
        .filter(|cookie| cookie.name() == COOKIE_NAME && !cookie.value().is_empty())
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

pub fn remove_auth_cookie(cookies: &Cookies, headers: &HeaderMap, state: &AppState) {
    for cookie in auth_cookies(headers) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use http::HeaderValue;
    use std::{fs, path::Path, time::Duration};

    #[tokio::test]
    async fn expired_cookie_does_not_mask_a_valid_session() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let config_path =
            std::env::temp_dir().join(format!("dashboard-auth-test-{}.toml", std::process::id()));
        fs::write(
            &config_path,
            format!(
                r#"
log_level = "error"
static_folder = {:?}
users_db = {:?}
cookie_domain = "example.test"
secure_cookies = true
cookie_duration = 1
[access_rules]
[external_links]
[admin_commands]
[admin_commands.hosts]
[admin_commands.panels]
[admin_commands.commands]
"#,
                root.join("package/static"),
                root.join("examples/users_db.txt"),
            ),
        )
        .unwrap();
        let config = config::AppConfig::from_file(&config_path);
        fs::remove_file(&config_path).unwrap();
        let state = AppState::init(config.unwrap());

        let sessions = state.get_sessions();
        let fresh = sessions.create("desktop", Duration::from_secs(60));
        let expired = sessions.create("desktop", Duration::ZERO);
        let mut headers = HeaderMap::new();
        headers.insert(
            COOKIE,
            HeaderValue::from_str(&format!("AuthUser={fresh}; AuthUser={expired}")).unwrap(),
        );
        assert_eq!(
            identify_user_with_cookie(&headers, &state).await,
            Ok(Some("desktop".to_string())),
        );

        let other = sessions.create("desktop", Duration::from_secs(60));
        headers.append(
            COOKIE,
            HeaderValue::from_str(&format!("AuthUser={other}")).unwrap(),
        );
        remove_auth_cookie(&Cookies::default(), &headers, &state);
        assert_eq!(identify_user_with_cookie(&headers, &state).await, Err(()));
    }
}
