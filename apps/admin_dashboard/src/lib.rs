use admin_cmd::router_admin_command;
use askama::Template;
use axum::{
    Router,
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::get,
};
use state::AppState;

mod admin_cmd;
mod templates;

use templates::AdminPanels;

pub fn router(State(state): State<AppState>) -> Router<AppState> {
    Router::new()
        .route(
            "/",
            get(move || async move {
                let panels = state.get_admin_commands().get_panels_with_commands();
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
        .route("/command", get(router_admin_command))
}
