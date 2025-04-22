mod models;
mod routes;
mod templates;

// Re-export the main components
pub use routes::{auth_routes, start_cache_cleanup};
