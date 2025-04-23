use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
};
use state::AppState;
use tower_cookies::Cookies;

use auth::{LOGIN_PATH, identify_user_with_cookie, set_redirect_cookie};

// Only check for subdomains
pub async fn check(
    State(state): State<AppState>,
    cookies: Cookies,
    headers: axum::http::HeaderMap,
) -> Response {
    let username = identify_user_with_cookie(&cookies, &state).await;

    // Get subdomain from headers
    let subdomain = headers
        .get("x-subdomain")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");

    if subdomain.is_empty() {
        tracing::debug!("No subdomain provided in request");
        return (StatusCode::BAD_REQUEST, "No subdomain provided".to_string()).into_response();
    }

    if state.is_subdomain_allowed(subdomain, username.as_deref()) {
        tracing::debug!(
            "User {:?} is allowed to access subdomain {}",
            username,
            subdomain
        );
        let mut response = StatusCode::OK.into_response();
        if let Some(username) = username {
            response
                .headers_mut()
                .insert("X-Authenticated-User", username.parse().unwrap());
        }
        return response;
    } else {
        tracing::debug!(
            "User {:?} is NOT allowed to access subdomain {}",
            username,
            subdomain
        );

        // Get the original URI if available
        let original_uri = headers
            .get("x-original-uri")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("/");

        // Construct the full URL to redirect back to after login
        let redirect_url = format!("https://{}.nginx.lan{}", subdomain, original_uri);

        // Set the redirect cookie
        set_redirect_cookie(&cookies, &state, &redirect_url);

        // Return a redirect to the login page
        return Redirect::to(LOGIN_PATH).into_response();
    }
}
