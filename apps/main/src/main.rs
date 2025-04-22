use axum_server::Handle;
use clap::Parser;
use config::AppConfig;
use rate_limiter::RateLimiter;
use routes::get_router;
use signal_handlers::spawn_sighup_watcher;
use state::AppState;
use std::net::SocketAddr;

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod cli_args;
mod routes;
mod signal_handlers;
mod templates;

use cli_args::Args;

#[tokio::main]
async fn main() {
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
        match cli_user_management::manage_users(&config.get_users_file()) {
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
            config.get_log_level().to_string(),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Start the server
    let addr = SocketAddr::from((config.get_server_address(), config.get_server_port()));

    // Create the auth state
    let app_state = AppState::new(RateLimiter::new(None), config);
    let app = get_router(axum::extract::State(app_state.clone()));

    tracing::info!("listening on {}", addr);

    spawn_sighup_watcher(app_state);

    axum_server::bind(addr)
        .handle(Handle::new())
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
}
