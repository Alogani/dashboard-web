use admin_cmd::router_admin_command;
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

mod admin_cmd;
mod templates;

use templates::AdminPanels;

pub fn router(State(state): State<AppState>) -> Router<AppState> {
    let rate_limiter = RateLimiter::new(Some(500));
    let state_clone = state.clone();
    Router::new()
        .route(
            "/",
            get(move || async move {
                let panels = state_clone.get_admin_commands().get_panels_with_commands();
                let template = AdminPanels { panels };
                match template.render() {
                    Ok(html) => Html(html).into_response(),
                    Err(err) => {
                        tracing::error!("Template error: {}", err);
                        StatusCode::INTERNAL_SERVER_ERROR.into_response()
                    }
                }
            }),
        )
        .route("/cmd/{cmd_name}", get(router_admin_command))
        .with_state((state, rate_limiter))
}
