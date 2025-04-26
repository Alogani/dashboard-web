use auth::remove_auth_cookie;
use axum::response::{IntoResponse, Redirect, Response};

use tower_cookies::Cookies;

pub async fn logout(cookies: Cookies) -> Response {
    remove_auth_cookie(&cookies);

    Redirect::to("/auth/login").into_response()
}
