use admin_command::router_admin_command;
use askama::Template;
use axum::{
    Router,
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::get,
};
use state::AppState;

mod admin_command;
mod templates;

use crate::templates::RouterAdminLanding;

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/",
            get(|| async {
                let template = RouterAdminLanding;
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
