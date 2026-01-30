use axum::{
    body::Body,
    http::Request,
    middleware::Next,
    response::{Redirect, Response},
};
use state::AppState;
use tower_cookies::Cookies;
use utils::with_nocache;

use crate::{AUTH_ROUTES, LOGIN_PATH, auth_cookie::*, redirect_cookie::set_redirect_cookie};

// Middleware to check if a user is authenticated and has access to a route
pub async fn auth_middleware(
    cookies: Cookies,
    state: AppState,
    req: Request<Body>,
    next: Next,
) -> Response {
    let path = req.uri().path().to_string();
    tracing::debug!("Auth middleware checking path: {}", path);

    if path.starts_with(AUTH_ROUTES) {
        tracing::debug!("Skipping authentication for {}", path);
        return next.run(req).await;
    }

    let app_config = state.get_app_config();

    // Check if user is authenticated
    match identify_user_with_cookie(&cookies, &state).await {
        Ok(Some(username)) => {
            // Check if user has access to this route
            if app_config.is_access_allowed(None, &path, Some(&username)) {
                tracing::debug!("User {} has access to {}", username, path);
                return next.run(req).await;
            } else {
                tracing::debug!("User {} does NOT have access to {}", username, path);
            }
        }
        Ok(None) => tracing::debug!("No user cookie found"),
        Err(_) => tracing::debug!("Error identifying user with cookie"),
    }

    // Check if unauthenticated users have access to this route
    if app_config.is_access_allowed(None, &path, None) {
        tracing::debug!("Allowing unauthenticated access to {}", path);
        return next.run(req).await;
    }

    // If not authenticated or not authorized, redirect to login
    tracing::debug!("Redirecting to login page");
    set_redirect_cookie(&cookies, &state, (None, path));

    with_nocache!(Redirect::to(LOGIN_PATH))
}
