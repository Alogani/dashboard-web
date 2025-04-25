use app_errors::AppError;
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
