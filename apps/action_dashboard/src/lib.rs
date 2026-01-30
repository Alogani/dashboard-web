use admin_executor::execute_admin_action;
use askama::Template;
use axum::{
    Router,
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::get,
};
use limiters_middleware::RateLimiter;
use state::AppState;

mod admin_executor;
mod templates;

use templates::AdminConsoleView;

pub fn router(State(state): State<AppState>) -> Router<AppState> {
    let rate_limiter = RateLimiter::new(Some(500), None);
    let state_clone = state.clone();
    Router::new()
        .route(
            "/",
            get(move || async move {
                let template = AdminConsoleView {
                    console: state_clone.get_admin_commands(),
                };
                match template.render() {
                    Ok(html) => {
                        let mut res = Html(html).into_response();
                        res.headers_mut().insert(
                            http::header::CACHE_CONTROL,
                            http::HeaderValue::from_static("no-store, no-cache, must-revalidate"),
                        );
                        res.headers_mut()
                            .insert(http::header::VARY, http::HeaderValue::from_static("Cookie"));
                        res
                    }
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
