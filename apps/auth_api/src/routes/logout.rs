use axum::response::{IntoResponse, Redirect, Response};

use tower_cookies::{Cookie, Cookies};

pub async fn logout(cookies: Cookies) -> Response {
    // Remove the auth cookie
    let mut cookie = Cookie::new("AuthUser", "");
    cookie.set_path("/");
    cookies.remove(cookie);

    Redirect::to("/auth/login").into_response()
}
