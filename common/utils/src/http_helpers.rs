use std::net::SocketAddr;

use http::HeaderMap;
use url::Url;

/// Canonicalizes a request target (path, path + query, or absolute URL) so it can be
/// compared against access rules.
///
/// The query string and fragment are dropped, percent-encoding is decoded once,
/// backslashes are treated as separators, empty and `.` segments are removed and a
/// trailing slash is kept. Returns `None` when the path cannot be safely canonicalized
/// (invalid encoding, non UTF-8, control characters or `..` segments): callers must
/// then deny access, as the upstream service could interpret the path differently.
pub fn normalize_request_path(target: &str) -> Option<String> {
    let path = if target.starts_with('/') {
        target
            .split(['?', '#'])
            .next()
            .unwrap_or_default()
            .to_string()
    } else {
        Url::parse(target).ok()?.path().to_string()
    };

    let decoded = String::from_utf8(percent_decode(&path)?).ok()?;
    if decoded.chars().any(|c| c.is_control()) {
        return None;
    }
    let decoded = decoded.replace('\\', "/");

    let mut segments = Vec::new();
    for segment in decoded.split('/') {
        match segment {
            "" | "." => continue,
            ".." => return None,
            _ => segments.push(segment),
        }
    }

    let mut normalized = format!("/{}", segments.join("/"));
    if !segments.is_empty() && (decoded.ends_with('/') || decoded.ends_with("/.")) {
        normalized.push('/');
    }
    Some(normalized)
}

fn percent_decode(input: &str) -> Option<Vec<u8>> {
    let bytes = input.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' {
            let hex = bytes.get(i + 1..i + 3)?;
            let value = u8::from_str_radix(std::str::from_utf8(hex).ok()?, 16).ok()?;
            decoded.push(value);
            i += 3;
        } else {
            decoded.push(bytes[i]);
            i += 1;
        }
    }
    Some(decoded)
}

/// Checks that a redirection target stays on the current host: it must be an absolute
/// path that cannot be interpreted as a protocol-relative URL (`//evil.com`, `/\evil.com`)
/// and must only contain printable ASCII characters.
pub fn is_safe_redirect_path(path: &str) -> bool {
    path.starts_with('/')
        && !path.starts_with("//")
        && !path.starts_with("/\\")
        && path.bytes().all(|b| b.is_ascii_graphic())
}

/// Checks that a subdomain only contains characters valid in a host name.
pub fn is_valid_subdomain(subdomain: &str) -> bool {
    !subdomain.is_empty()
        && subdomain.len() <= 253
        && subdomain
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'.' || b == b'_')
        && !subdomain.starts_with('.')
        && !subdomain.ends_with('.')
}

/// Returns the IP of the client.
///
/// Forwarding headers are only trusted when the request comes from the loopback
/// interface (i.e. from the local reverse proxy), otherwise any client could spoof
/// its address to bypass rate limiting or to get another IP banned by fail2ban.
/// `X-Real-IP` is preferred, then the last entry of `X-Forwarded-For` (the one
/// appended by the reverse proxy itself).
pub fn get_client_ip(headers: &HeaderMap, socket_addr: &SocketAddr) -> String {
    if socket_addr.ip().is_loopback() {
        let real_ip = headers
            .get("X-Real-IP")
            .and_then(|h| h.to_str().ok())
            .map(str::trim)
            .filter(|ip| !ip.is_empty());
        let forwarded_for = || {
            headers
                .get("X-Forwarded-For")
                .and_then(|h| h.to_str().ok())
                .and_then(|h| h.rsplit(',').next())
                .map(str::trim)
                .filter(|ip| !ip.is_empty())
        };
        if let Some(ip) = real_ip.or_else(forwarded_for) {
            return ip.to_string();
        }
    }

    socket_addr.ip().to_string()
}

/// Checks that a state changing request was issued by a page of the same host,
/// using the `Origin` header (or `Referer` as a fallback). Requests without any of
/// those headers are rejected, browsers always send at least one of them on POST.
pub fn is_same_origin_request(headers: &HeaderMap) -> bool {
    let host = match headers
        .get(http::header::HOST)
        .and_then(|h| h.to_str().ok())
    {
        Some(host) => host_without_port(host),
        None => return false,
    };

    let source = headers
        .get(http::header::ORIGIN)
        .or_else(|| headers.get(http::header::REFERER))
        .and_then(|h| h.to_str().ok());

    match source.and_then(|source| Url::parse(source).ok()) {
        Some(url) => url
            .host_str()
            .is_some_and(|source_host| source_host.eq_ignore_ascii_case(host)),
        None => false,
    }
}

fn host_without_port(host: &str) -> &str {
    if host.starts_with('[') {
        // IPv6 literal, keep the brackets as returned by Url::host_str
        host.split_inclusive(']').next().unwrap_or(host)
    } else {
        host.split(':').next().unwrap_or(host)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use http::HeaderValue;

    #[test]
    fn test_normalize_request_path() {
        assert_eq!(normalize_request_path("/").as_deref(), Some("/"));
        assert_eq!(normalize_request_path("/admin").as_deref(), Some("/admin"));
        assert_eq!(
            normalize_request_path("/admin/").as_deref(),
            Some("/admin/")
        );
        assert_eq!(
            normalize_request_path("/admin?x=1").as_deref(),
            Some("/admin")
        );
        assert_eq!(
            normalize_request_path("/admin#x").as_deref(),
            Some("/admin")
        );
        assert_eq!(
            normalize_request_path("/%61dmin").as_deref(),
            Some("/admin")
        );
        assert_eq!(normalize_request_path("//admin").as_deref(), Some("/admin"));
        assert_eq!(
            normalize_request_path("/./admin").as_deref(),
            Some("/admin")
        );
        assert_eq!(
            normalize_request_path("/a\\admin").as_deref(),
            Some("/a/admin")
        );
        assert_eq!(
            normalize_request_path("/cmd/router%5Fpoweroff").as_deref(),
            Some("/cmd/router_poweroff")
        );
        assert_eq!(
            normalize_request_path("https://host.lan/admin?x").as_deref(),
            Some("/admin")
        );
        assert_eq!(normalize_request_path("/a/../admin"), None);
        assert_eq!(normalize_request_path("/a/%2e%2e/admin"), None);
        assert_eq!(normalize_request_path("/a/%2E%2E%2Fadmin"), None);
        assert_eq!(normalize_request_path("/admin%"), None);
        assert_eq!(normalize_request_path("/admin%zz"), None);
        assert_eq!(normalize_request_path("/admin%00"), None);
        assert_eq!(normalize_request_path("/admin%ff"), None);
        assert_eq!(normalize_request_path("admin"), None);
    }

    #[test]
    fn test_is_safe_redirect_path() {
        assert!(is_safe_redirect_path("/"));
        assert!(is_safe_redirect_path("/admin?x=1"));
        assert!(!is_safe_redirect_path("//evil.com"));
        assert!(!is_safe_redirect_path("/\\evil.com"));
        assert!(!is_safe_redirect_path("https://evil.com"));
        assert!(!is_safe_redirect_path("/a b"));
        assert!(!is_safe_redirect_path("/a\r\nSet-Cookie: x"));
    }

    #[test]
    fn test_is_valid_subdomain() {
        assert!(is_valid_subdomain("vaultwarden"));
        assert!(is_valid_subdomain("my-app.coding"));
        assert!(!is_valid_subdomain(""));
        assert!(!is_valid_subdomain("evil.com/"));
        assert!(!is_valid_subdomain("evil.com#"));
        assert!(!is_valid_subdomain("user@evil.com"));
        assert!(!is_valid_subdomain(".evil"));
    }

    #[test]
    fn test_get_client_ip() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "X-Forwarded-For",
            HeaderValue::from_static("6.6.6.6, 1.2.3.4"),
        );
        let proxy: SocketAddr = "127.0.0.1:1234".parse().unwrap();
        let remote: SocketAddr = "5.5.5.5:1234".parse().unwrap();

        assert_eq!(get_client_ip(&headers, &proxy), "1.2.3.4");
        assert_eq!(get_client_ip(&headers, &remote), "5.5.5.5");

        headers.insert("X-Real-IP", HeaderValue::from_static("9.9.9.9"));
        assert_eq!(get_client_ip(&headers, &proxy), "9.9.9.9");
        assert_eq!(get_client_ip(&headers, &remote), "5.5.5.5");
    }

    #[test]
    fn test_is_same_origin_request() {
        let mut headers = HeaderMap::new();
        headers.insert(http::header::HOST, HeaderValue::from_static("nginx.lan"));
        assert!(!is_same_origin_request(&headers));

        headers.insert(
            http::header::ORIGIN,
            HeaderValue::from_static("https://nginx.lan"),
        );
        assert!(is_same_origin_request(&headers));

        headers.insert(
            http::header::ORIGIN,
            HeaderValue::from_static("https://forgejo.nginx.lan"),
        );
        assert!(!is_same_origin_request(&headers));

        headers.insert(http::header::ORIGIN, HeaderValue::from_static("null"));
        assert!(!is_same_origin_request(&headers));

        headers.remove(http::header::ORIGIN);
        headers.insert(
            http::header::REFERER,
            HeaderValue::from_static("https://nginx.lan/action_dashboard"),
        );
        assert!(is_same_origin_request(&headers));

        headers.insert(
            http::header::HOST,
            HeaderValue::from_static("127.0.0.1:8080"),
        );
        headers.insert(
            http::header::REFERER,
            HeaderValue::from_static("http://127.0.0.1:8080/action_dashboard"),
        );
        assert!(is_same_origin_request(&headers));
    }
}
