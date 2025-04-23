use askama::Template;
use auth::auth_middleware;
use auth_api::auth_routes;
use axum::{Router, extract::State, response::Html, routing::get};
use state::AppState;
use tower_http::{services::ServeDir, trace::TraceLayer};

use crate::templates::LandingPage;

pub fn get_router(State(app_state): State<AppState>) -> Router {
    let landing_page = LandingPage {
        external_links: app_state.get_external_links().clone(),
    };

    let app_state_clone = app_state.clone();
    Router::new()
        .route("/", get(Html(landing_page.render().unwrap())))
        .nest(
            "/action_dashboard",
            action_dashboard::router(axum::extract::State(app_state.clone())),
        )
        .nest(
            "/auth",
            auth_routes(axum::extract::State(app_state.clone())),
        )
        .nest_service("/static", ServeDir::new(app_state.get_static_folder()))
        .layer(axum::middleware::from_fn(move |cookies, req, next| {
            let app_state = app_state_clone.clone();
            auth_middleware(cookies, app_state, req, next)
        }))
        .layer(tower_cookies::CookieManagerLayer::new())
        .layer(TraceLayer::new_for_http())
        .with_state(app_state)
}
