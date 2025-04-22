use askama::Template;
use auth::AuthState;
use auth::auth_routes;
use axum::Router;
use axum::response::Html;
use axum::routing::get;
use axum_server::Handle;
use clap::Parser;
use common::RateLimiter;
use config::{AppConfig, UsersConfig};
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::RwLock;
use tower::ServiceBuilder;
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod cli;
mod templates;
mod user_management;

use auth::auth_middleware;
use cli::Args;
use templates::LandingPage;

#[tokio::main]
async fn main() {
    auth::start_cache_cleanup();

    let args = Args::parse();

    // Load the configuration
    let config = match AppConfig::from_file(&args.config) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Error loading configuration: {}", e);
            std::process::exit(1);
        }
    };

    // If the user wants to manage users, do that and exit
    if args.manage_users {
        match user_management::manage_users(&config.users_file) {
            Ok(_) => return,
            Err(e) => {
                eprintln!("Error managing users: {}", e);
                std::process::exit(1);
            }
        }
    }

    // Set up logging based on configuration
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            config.log_level.to_string(),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load users and routes configurations
    let users_config = match UsersConfig::from_file(&config.users_file) {
        Ok(config) => Arc::new(RwLock::new(config)),
        Err(e) => {
            eprintln!("Error loading users configuration: {}", e);
            std::process::exit(1);
        }
    };

    let landing_page = LandingPage {
        external_links: config.external_links.clone(),
    };

    let app_config = Arc::new(RwLock::new(config));

    // Create the auth state
    let auth_state = AuthState {
        rate_limiter: RateLimiter::new(None),
        app_config: app_config.clone(),
        users_config: users_config.clone(),
    };

    // Create the router with state and middleware using ServiceBuilder
    let auth_state_clone = auth_state.clone();
    let app = Router::new()
        .route("/", get(Html(landing_page.render().unwrap())))
        .nest("/admin_dashboard", admin_dashboard::router())
        .nest(
            "/auth",
            auth_routes(app_config.clone(), users_config.clone()),
        )
        .nest_service("/static", ServeDir::new("static"))
        .layer(
            ServiceBuilder::new()
                .layer(tower_cookies::CookieManagerLayer::new())
                .layer(TraceLayer::new_for_http())
                .layer(axum::middleware::from_fn(move |cookies, req, next| {
                    let auth_state_clone = auth_state_clone.clone();
                    auth_middleware(cookies, auth_state_clone, req, next)
                })),
        )
        .with_state(auth_state);

    // Start the server
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("listening on {}", addr);

    let handle = Handle::new();

    axum_server::bind(addr)
        .handle(handle)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
}
