use axum::{
    body::Body,
    http::Request,
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
};
use tower_cookies::Cookies;

use crate::models::AuthState;

// Middleware to check if a user is authenticated and has access to a route
pub async fn auth_middleware(
    cookies: Cookies,
    state: AuthState,
    req: Request<Body>,
    next: Next,
) -> Response {
    let path = req.uri().path().to_string();
    tracing::debug!("Auth middleware checking path: {}", path);

    // Always allow access to login page and static resources
    if path.starts_with("/auth/") || path.starts_with("/static/") {
        tracing::debug!("Allowing access to login or static resource");
        return next.run(req).await;
    }

    let routes_config = state.routes_config.read().await;

    // Check if user is authenticated
    if let Some(username) = get_user_from_cookie(&state, cookies).await {
        // Check if user has access to this route
        if routes_config.is_route_allowed(&path, Some(&username)) {
            tracing::debug!("User {} has access to {}", username, path);
            return next.run(req).await;
        } else {
            tracing::debug!("User {} does NOT have access to {}", username, path);
        }
    } else {
        tracing::debug!("No user cookie found");

        // Check if unauthenticated users have access to this route
        if routes_config.is_route_allowed(&path, None) {
            tracing::debug!("Allowing unauthenticated access to {}", path);
            return next.run(req).await;
        }
    }

    // If not authenticated or not authorized, redirect to login
    tracing::debug!("Redirecting to login page");
    Redirect::to("/auth/login").into_response()
}

pub(crate) async fn get_user_from_cookie(state: &AuthState, cookies: Cookies) -> Option<String> {
    if let Some(cookie) = cookies.get("AuthUser") {
        state
            .users_config
            .read()
            .await
            .get_username_from_hash(cookie.value())
    } else {
        None
    }
}
