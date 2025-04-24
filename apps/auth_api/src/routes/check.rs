use axum::{
    extract::State,
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use limiters_middleware::RateLimiter;
use state::AppState;
use tower_cookies::Cookies;

use auth::{identify_user_with_cookie, set_redirect_cookie};

// Only check for subdomains
pub async fn check(
    State((state, _)): State<(AppState, RateLimiter)>,
    cookies: Cookies,
    headers: HeaderMap,
) -> Response {
    let username = identify_user_with_cookie(&cookies, &state).await;

    // Get subdomain from headers
    let subdomain = headers
        .get("x-subdomain")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");

    if subdomain.is_empty() {
        tracing::debug!("No subdomain provided in request");
        return (StatusCode::BAD_REQUEST, "No subdomain provided").into_response();
    }

    if state.is_subdomain_allowed(subdomain, username.as_deref()) {
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

        // Get the original URI if available
        let original_uri = headers
            .get("x-original-uri")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("/");

        // Construct the full URL to redirect back to after login
        let redirect_url = format!("https://{}.nginx.lan{}", subdomain, original_uri);

        // Set the redirect cookie
        set_redirect_cookie(&cookies, &state, &redirect_url);

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
