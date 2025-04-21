use crate::{middleware::get_user_from_cookie, models::AuthState};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use std::collections::HashMap;
use std::sync::LazyLock;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tower_cookies::Cookies;

// Cache structure to store authentication results
static AUTH_CACHE: LazyLock<RwLock<HashMap<String, (String, Instant)>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

// Cache TTL in seconds
const CACHE_TTL: u64 = 300; // 5 minutes

pub async fn check_cookie(
    State(state): State<AuthState>,
    cookies: Cookies,
    headers: axum::http::HeaderMap,
) -> Response {
    // Get auth token from cookies for cache key
    let auth_token = cookies.get("auth_token").map(|c| c.value().to_string());

    // Get subdomain from headers
    let subdomain = headers
        .get("x-subdomain")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");

    if subdomain.is_empty() {
        tracing::debug!("No subdomain provided in request");
        return (
            StatusCode::BAD_REQUEST,
            "No subdomain provided".to_string(),
        )
            .into_response();
    }

    // Create cache key from token and subdomain
    if let Some(token) = &auth_token {
        let cache_key = format!("{}:{}", token, subdomain);

        // Check cache first
        if let Some((username, timestamp)) = AUTH_CACHE.read().await.get(&cache_key) {
            if timestamp.elapsed() < Duration::from_secs(CACHE_TTL) {
                // Get app config to check subdomain permissions
                let app_config = state.app_config.read().await;
                
                // Cache hit, check if user is allowed for this subdomain
                if app_config.is_subdomain_allowed(subdomain, Some(username)) {
                    tracing::debug!("Cache hit: User {} is allowed for subdomain {}", username, subdomain);

                    // Add username to response headers for nginx
                    let mut response = StatusCode::OK.into_response();
                    response
                        .headers_mut()
                        .insert("X-Authenticated-User", username.parse().unwrap());
                    return response;
                } else {
                    tracing::debug!("Cache hit: User {} is NOT allowed for subdomain {}", username, subdomain);
                    return (
                        StatusCode::FORBIDDEN,
                        "You don't have permission to access this resource".to_string(),
                    )
                        .into_response();
                }
            }
        }
    }

    // Cache miss or expired, perform full authentication
    if let Some(username) = get_user_from_cookie(&state, cookies).await {
        // Get app config to check subdomain permissions
        let app_config = state.app_config.read().await;
        
        // Check if the user is allowed to access this subdomain
        if app_config.is_subdomain_allowed(subdomain, Some(&username)) {
            tracing::debug!("User {} is allowed to access subdomain {}", username, subdomain);

            // Store in cache
            if let Some(token) = auth_token {
                let cache_key = format!("{}:{}", token, subdomain);
                AUTH_CACHE
                    .write()
                    .await
                    .insert(cache_key, (username.clone(), Instant::now()));
            }

            // Add username to response headers for nginx
            let mut response = StatusCode::OK.into_response();
            response
                .headers_mut()
                .insert("X-Authenticated-User", username.parse().unwrap());
            return response;
        } else {
            tracing::debug!("User {} is NOT allowed to access subdomain {}", username, subdomain);
            
            // Cache negative result too
            if let Some(token) = auth_token {
                let cache_key = format!("{}:{}", token, subdomain);
                AUTH_CACHE
                    .write()
                    .await
                    .insert(cache_key, (username.clone(), Instant::now()));
            }
            
            return (
                StatusCode::FORBIDDEN,
                "You don't have permission to access this resource".to_string(),
            )
                .into_response();
        }
    } else {
        tracing::debug!("No valid authentication cookie found");
        
        // Cache failed authentication attempts to prevent brute force
        if let Some(token) = auth_token {
            let cache_key = format!("{}:{}", token, subdomain);
            // Use a special marker for unauthenticated users
            AUTH_CACHE
                .write()
                .await
                .insert(cache_key, ("__unauthenticated__".to_string(), Instant::now()));
        }
        
        return (
            StatusCode::UNAUTHORIZED,
            "Invalid authentication".to_string(),
        )
            .into_response();
    }
}

// Add a periodic cache cleanup task
pub fn start_cache_cleanup() {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(60)); // Run every minute
        loop {
            interval.tick().await;
            tracing::debug!("Running auth cache cleanup");
            
            let before_count = AUTH_CACHE.read().await.len();
            AUTH_CACHE
                .write()
                .await
                .retain(|_, (_, timestamp)| timestamp.elapsed() < Duration::from_secs(CACHE_TTL));
            let after_count = AUTH_CACHE.read().await.len();
            
            if before_count != after_count {
                tracing::debug!("Auth cache cleanup: removed {} entries", before_count - after_count);
            }
        }
    });
}
