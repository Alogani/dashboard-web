use state::AppState;
use tower_cookies::{Cookie, Cookies};

const COOKIE_NAME: &str = "AuthRedirect";

pub fn set_redirect_cookie(cookies: &Cookies, state: &AppState, path: &str) {
    let cookie = Cookie::build((COOKIE_NAME, path.to_string()))
        .path("/")
        .http_only(true)
        .secure(state.use_secure_cookies())
        .same_site(tower_cookies::cookie::SameSite::Strict);
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

pub fn get_redirect_cookie(cookies: &Cookies) -> Option<String> {
    cookies.get(COOKIE_NAME).map(|cookie| {
        let path = cookie.value().to_string();
        cookies.remove(Cookie::new(COOKIE_NAME, "")); // Remove the cookie after reading
        path
    })
}
