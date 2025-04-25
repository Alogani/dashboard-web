use axum::{
    extract::State,
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use limiters_middleware::RateLimiter;
use state::AppState;
use tower_cookies::Cookies;

use auth::{identify_user_with_cookie, set_redirect_cookie};
use utils::http_helpers::extract_path_from_url;

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

    // Get the original URI if available
    let original_uri = headers
        .get("x-original-uri")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("/");

    // Extract path from the original URI
    let path = extract_path_from_url(original_uri).unwrap_or(original_uri.to_string());

    if state.is_access_allowed(Some(&subdomain), &path, username.as_deref()) {
        tracing::debug!(
            "User {:?} is allowed to access subdomain {}",
            username,
            subdomain
        );
        let mut response = StatusCode::OK.into_response();
        if let Some(username) = username {
            response.headers_mut().insert(
                "X-Authenticated-User",
                HeaderValue::from_str(&username).unwrap(),
            );
        }
        response
    } else {
        tracing::debug!(
            "User {:?} is NOT allowed to access subdomain {}",
            username,
            subdomain
        );

        // Set the redirect cookie
        set_redirect_cookie(&cookies, &state, (Some(subdomain.to_string()), path));

        // Return an unauthorized status
        //StatusCode::UNAUTHORIZED.into_response()
        let mut response = StatusCode::UNAUTHORIZED.into_response();

        // Manually add Set-Cookie header
        if let Some(cookie) = cookies.get("AuthRedirect2") {
            response.headers_mut().insert(
                header::SET_COOKIE,
                HeaderValue::from_str(&cookie.to_string()).unwrap(),
            );
        }

        // Log response headers
        tracing::debug!("Response headers: {:?}", response.headers());
        response
    }
}
