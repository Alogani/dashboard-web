use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use std::fmt;

#[derive(Debug)]
pub enum AppError {
    AuthenticationError(String),
    AuthorizationError(String),
    ConfigurationError(String),
    DatabaseError(String),
    ExternalServiceError(String),
    InputValidationError(String),
    NotFoundError(String),
    RateLimitExceeded(String),
    ServerError(String),
    BcryptError(bcrypt::BcryptError),
    IoError(std::io::Error),
    TomlDeError(toml::de::Error),
    TomlSerError(toml::ser::Error),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::AuthenticationError(msg) => write!(f, "Authentication failed: {}", msg),
            AppError::AuthorizationError(msg) => write!(f, "Not authorized: {}", msg),
            AppError::ConfigurationError(msg) => write!(f, "Configuration error: {}", msg),
            AppError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            AppError::ExternalServiceError(msg) => write!(f, "External service error: {}", msg),
            AppError::InputValidationError(msg) => write!(f, "Invalid input: {}", msg),
            AppError::NotFoundError(msg) => write!(f, "Resource not found: {}", msg),
            AppError::RateLimitExceeded(msg) => write!(f, "Rate limit exceeded: {}", msg),
            AppError::ServerError(msg) => write!(f, "Internal server error: {}", msg),
            AppError::BcryptError(err) => write!(f, "Bcrypt error: {}", err),
            AppError::IoError(err) => write!(f, "IO error: {}", err),
            AppError::TomlDeError(err) => write!(f, "TOML error: {}", err),
            AppError::TomlSerError(err) => write!(f, "TOML error: {}", err),
        }
    }
}

impl std::error::Error for AppError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AppError::BcryptError(err) => Some(err),
            AppError::IoError(err) => Some(err),
            AppError::TomlDeError(err) => Some(err),
            AppError::TomlSerError(err) => Some(err),
            _ => None,
        }
    }
}

impl From<bcrypt::BcryptError> for AppError {
    fn from(err: bcrypt::BcryptError) -> Self {
        AppError::BcryptError(err)
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::IoError(err)
    }
}

impl From<toml::de::Error> for AppError {
    fn from(err: toml::de::Error) -> Self {
        AppError::TomlDeError(err)
    }
}

impl From<toml::ser::Error> for AppError {
    fn from(err: toml::ser::Error) -> Self {
        AppError::TomlSerError(err)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::AuthenticationError(_) => (StatusCode::UNAUTHORIZED, "Authentication failed"),
            AppError::AuthorizationError(_) => (StatusCode::FORBIDDEN, "Not authorized"),
            AppError::ConfigurationError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Configuration error")
            }
            AppError::DatabaseError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Database error"),
            AppError::ExternalServiceError(_) => {
                (StatusCode::BAD_GATEWAY, "External service error")
            }
            AppError::InputValidationError(_) => (StatusCode::BAD_REQUEST, "Invalid input"),
            AppError::NotFoundError(_) => (StatusCode::NOT_FOUND, "Resource not found"),
            AppError::RateLimitExceeded(_) => {
                (StatusCode::TOO_MANY_REQUESTS, "Rate limit exceeded")
            }
            AppError::ServerError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            }
            AppError::TomlDeError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "TOML deserialization error",
            ),
            AppError::TomlSerError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "TOML serialization error",
            ),
            AppError::BcryptError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Bcrypt error"),
            AppError::IoError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "IO error"),
        };

        (status, error_message).into_response()
    }
}
