use admin_executor::execute_admin_action;
use askama::Template;
use axum::{
    Router,
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::get,
};
use rate_limiter::RateLimiter;
use state::AppState;

mod admin_executor;
mod templates;

use templates::AdminConsoleView;

pub fn router(State(state): State<AppState>) -> Router<AppState> {
    let rate_limiter = RateLimiter::new(Some(500));
    let state_clone = state.clone();
    Router::new()
        .route(
            "/",
            get(move || async move {
                let template = AdminConsoleView {
                    console: state_clone.get_admin_commands(),
                };
                match template.render() {
                    Ok(html) => Html(html).into_response(),
                    Err(err) => {
                        tracing::error!("Template error: {}", err);
                        StatusCode::INTERNAL_SERVER_ERROR.into_response()
                    }
                }
            }),
        )
        .route("/cmd/{cmd_name}", get(execute_admin_action))
        .with_state((state, rate_limiter))
}
