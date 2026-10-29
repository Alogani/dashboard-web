use axum::{
    extract::State,
    http::{HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};
use limiters_middleware::RateLimiter;
use state::AppState;
use tower_cookies::Cookies;

use auth::{identify_user_with_cookie, set_redirect_cookie};
use utils::http_helpers::is_valid_subdomain;

// Only check for subdomains
pub async fn check(
    State((state, _)): State<(AppState, RateLimiter)>,
    cookies: Cookies,
    headers: HeaderMap,
) -> Response {
    let username = identify_user_with_cookie(&cookies, &state)
        .await
        .unwrap_or(None);

    // Get subdomain from headers
    let subdomain = headers
        .get("x-subdomain")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");

    if subdomain.is_empty() {
        tracing::debug!("No subdomain provided in request");
        return (StatusCode::BAD_REQUEST, "No subdomain provided").into_response();
    }
    if !is_valid_subdomain(subdomain) {
        tracing::warn!("Invalid subdomain provided in request: {:?}", subdomain);
        return (StatusCode::BAD_REQUEST, "Invalid subdomain").into_response();
    }

    // Get the original URI (path and query) if available, it is canonicalized by the access check
    let original_uri = headers
        .get("x-original-uri")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("/");

    if state.is_access_allowed(Some(subdomain), original_uri, username.as_deref()) {
        tracing::debug!(
            "User {:?} is allowed to access subdomain {}",
            username,
            subdomain
        );
        let mut response = StatusCode::OK.into_response();
        if let Some(header_value) = username.and_then(|u| HeaderValue::from_str(&u).ok()) {
            response
                .headers_mut()
                .insert("X-Authenticated-User", header_value);
        }
        response
    } else {
        tracing::debug!(
            "User {:?} is NOT allowed to access subdomain {}",
            username,
            subdomain
        );

        set_redirect_cookie(
            &cookies,
            &state,
            (Some(subdomain.to_string()), original_uri.to_string()),
        );

        StatusCode::UNAUTHORIZED.into_response()
    }
}
