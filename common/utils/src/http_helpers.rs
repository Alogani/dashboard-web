use app_errors::AppError;
use tower_cookies::{Cookie, Cookies};
use url::Url;

pub fn extract_path_from_url(url_str: &str) -> Result<String, AppError> {
    match Url::parse(url_str) {
        Ok(url) => {
            let path = url.path();
            // If path is empty, return "/" as the root path
            if path.is_empty() {
                Ok("/".to_string())
            } else {
                // Include query parameters if present
                if let Some(query) = url.query() {
                    Ok(format!("{}?{}", path, query))
                } else {
                    Ok(path.to_string())
                }
            }
        }
        Err(_) => Err(AppError::ConfigurationError(
            format!(
                "Invalid URL format. Please provide a valid URL. Incompatible URL: {}.",
                url_str.to_string()
            )
            .into(),
        )),
    }
}

pub fn remove_cookie(cookies: &Cookies, cookie: &Option<Cookie>) {
    let cookie = if let Some(cookie) = cookie {
        cookie
    } else {
        return;
    };
    let new_cookie = Cookie::build((cookie.name().to_string(), ""))
        .path(cookie.path().map(String::from).unwrap_or("/".to_string())) // Use the original path if available
        .http_only(cookie.http_only().unwrap_or(true));
    let new_cookie = if let Some(secure) = cookie.secure() {
        new_cookie.secure(secure)
    } else {
        new_cookie
    };
    let new_cookie = if let Some(domain) = cookie.domain() {
        new_cookie.domain(domain.to_string())
    } else {
        new_cookie
    };
    let new_cookie = new_cookie.expires(time::OffsetDateTime::now_utc() - time::Duration::days(1));
    cookies.remove(new_cookie.into());
}
