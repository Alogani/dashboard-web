use state::AppState;
use url::Url;

// Extract subdomain from a host string (e.g., "incus" from "incus.nginx.lan")
pub fn extract_subdomain_from_host(host: &str) -> &str {
    host.split('.').next().unwrap_or("")
}

// Check if a user has access to a subdomain
pub fn is_user_allowed_subdomain(
    state: &AppState,
    subdomain: &str,
    username: Option<&str>,
) -> bool {
    if subdomain.is_empty() {
        return false;
    }

    state.is_subdomain_allowed(subdomain, username)
}

// Extract subdomain from a full URL
pub fn extract_subdomain_from_url(url_str: &str) -> Option<String> {
    if url_str.starts_with("http://") || url_str.starts_with("https://") {
        match Url::parse(url_str) {
            Ok(url) => {
                if let Some(host) = url.host_str() {
                    let subdomain = extract_subdomain_from_host(host);
                    if !subdomain.is_empty() {
                        return Some(subdomain.to_string());
                    }
                }
            }
            Err(_) => return None,
        }
    }
    None
}

// Create a full URL for a subdomain
pub fn create_subdomain_url(subdomain: &str, path: &str) -> String {
    format!("https://{}.nginx.lan{}", subdomain, path)
}
