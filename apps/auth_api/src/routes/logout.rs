use axum::response::{IntoResponse, Redirect, Response};

use tower_cookies::Cookies;

use crate::auth_cookie::clear_auth_cookie;

pub async fn logout(cookies: Cookies) -> Response {
    clear_auth_cookie(&cookies);

    Redirect::to("/auth/login").into_response()
}
