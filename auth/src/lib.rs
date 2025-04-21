mod middleware;
mod models;
mod routes;
mod templates;
mod utils;

// Re-export the main components
pub use middleware::auth_middleware;
pub use models::AuthState;
pub use routes::auth_routes;
pub use utils::generate_password_hash;
