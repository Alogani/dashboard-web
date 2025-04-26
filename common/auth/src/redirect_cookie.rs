use serde::{Deserialize, Serialize};
use state::AppState;
use tower_cookies::{Cookie, Cookies};
use utils::http_helpers::remove_cookie;

const COOKIE_NAME: &str = "AuthRedirect";

#[derive(Serialize, Deserialize)]
struct RedirectData {
    subdomain: Option<String>,
    path: String,
}

pub fn set_redirect_cookie(
    cookies: &Cookies,
    state: &AppState,
    redirection: (Option<String>, String),
) {
    tracing::trace!("Setting redirect cookie for redirection: {:?}", redirection);
    let redirect_data = RedirectData {
        subdomain: redirection.0,
        path: redirection.1,
    };
    let serialized = serde_json::to_string(&redirect_data).unwrap_or_else(|_| {
        tracing::error!("Failed to serialize redirect data");
        "{}".to_string()
    });

    let cookie = Cookie::build((COOKIE_NAME, serialized))
        .path("/")
        .http_only(true)
        .secure(state.use_secure_cookies())
        .same_site(tower_cookies::cookie::SameSite::Lax);
    let domain = state.get_cookie_domain().to_string();
    let cookie = if !domain.is_empty() {
        cookie.domain(domain)
    } else {
        cookie
    };

    // The cookie will be a session cookie (expires when the browser is closed)
    // by not setting an expiration time
    cookies.add(cookie.into());
}

pub fn consume_redirect_cookie(cookies: &Cookies) -> Option<(Option<String>, String)> {
    cookies.get(COOKIE_NAME).and_then(|cookie| {
        let serialized_value = cookie.value();
        remove_cookie(&cookies, &cookies.get(COOKIE_NAME));
        tracing::trace!("Cleared redirect cookie");

        match serde_json::from_str::<RedirectData>(serialized_value) {
            Ok(redirect_data) => Some((redirect_data.subdomain, redirect_data.path)),
            Err(err) => {
                tracing::error!("Failed to deserialize redirect cookie: {}", err);
                None
            }
        }
    })
}
