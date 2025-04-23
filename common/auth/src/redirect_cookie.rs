use tower_cookies::{Cookie, Cookies};

const COOKIE_NAME: &str = "auth_redirect";

pub fn set_redirect_cookie(cookies: &Cookies, path: &str) {
    let mut cookie = Cookie::new(COOKIE_NAME, path.to_string());
    cookie.set_path("/");
    cookie.set_http_only(true);
    cookie.set_secure(true);
    cookie.set_same_site(tower_cookies::cookie::SameSite::Strict);

    // The cookie will be a session cookie (expires when the browser is closed)
    // by not setting an expiration time

    cookies.add(cookie);
}

pub fn get_redirect_cookie(cookies: &Cookies) -> Option<String> {
    cookies.get(COOKIE_NAME).map(|cookie| {
        let path = cookie.value().to_string();
        cookies.remove(Cookie::new(COOKIE_NAME, "")); // Remove the cookie after reading
        path
    })
}
