use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};

use tower_cookies::Cookies;

use crate::{middleware::get_user_from_cookie, models::AuthState};

pub async fn check_cookie(
    State(state): State<AuthState>,
    cookies: Cookies,
    headers: axum::http::HeaderMap,
) -> Response {
    // Get the authenticated user from cookie
    if let Some(username) = get_user_from_cookie(&state, cookies).await {
        // Check if there's an X-Allowed-Users header from Nginx
        let allowed_users = if let Some(allowed) = headers.get("x-allowed-users") {
            if let Ok(allowed_str) = allowed.to_str() {
                allowed_str
            } else {
                "*" // Default to all users if header can't be parsed
            }
        } else {
            tracing::debug!("No X-Allowed-Users header found");
            return (
                StatusCode::UNAUTHORIZED,
                "Invalid authentication".to_string(),
            )
                .into_response();
        };

        // Get the host from the Host header
        let host = headers
            .get("host")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("unknown");

        // Parse the allowed users string (comma-separated list)
        let allowed_users_vec: Vec<&str> = allowed_users.split(',').map(|s| s.trim()).collect();

        // Check if the authenticated user is allowed
        if allowed_users == "*" || allowed_users_vec.contains(&username.as_str()) {
            tracing::debug!("User {} is allowed to access {}", username, host);
            StatusCode::OK.into_response()
        } else {
            tracing::debug!("User {} is NOT allowed to access {}", username, host);
            (
                StatusCode::FORBIDDEN,
                "You don't have permission to access this resource".to_string(),
            )
                .into_response()
        }
    } else {
        tracing::debug!("No valid authentication cookie found");
        (
            StatusCode::UNAUTHORIZED,
            "Invalid authentication".to_string(),
        )
            .into_response()
    }
}
