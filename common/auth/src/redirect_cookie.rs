use serde::{Deserialize, Serialize};
use state::AppState;
use tower_cookies::{Cookie, Cookies, cookie::CookieBuilder};

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

    cookies.add(build_cookie(serialized, state).into());
}

pub fn consume_redirect_cookie(
    cookies: &Cookies,
    state: &AppState,
) -> Option<(Option<String>, String)> {
    cookies.get(COOKIE_NAME).and_then(|cookie| {
        let serialized_value = cookie.value();
        cookies.remove(build_cookie("".to_string(), state).into());
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

fn build_cookie<'a>(value: String, state: &AppState) -> CookieBuilder<'a> {
    // The cookie will be a session cookie (expires when the browser is closed)
    // by not setting an expiration time
    let cookie = Cookie::build((COOKIE_NAME, value))
        .path("/")
        .http_only(true)
        .secure(state.use_secure_cookies())
        .same_site(tower_cookies::cookie::SameSite::Lax);
    let domain = state.get_cookie_domain().to_string();
    if !domain.is_empty() {
        cookie.domain(domain)
    } else {
        cookie
    }
}
