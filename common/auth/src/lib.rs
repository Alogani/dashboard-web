mod auth_cookie;
mod middleware;
pub mod redirect_cookie;

const AUTH_ROUTES: &str = "/auth/";
const LOGIN_PATH: &str = "/auth/login";

pub use auth_cookie::*;
pub use middleware::auth_middleware;
pub use redirect_cookie::{consume_redirect_cookie, set_redirect_cookie};
