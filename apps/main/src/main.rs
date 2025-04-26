use axum_server::Handle;
use clap::Parser;
use config::AppConfig;
use routes::get_router;
use state::AppState;
use std::net::SocketAddr;

mod cli_args;
mod routes;
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

    // Initialize the application state, this also initialize logging
    let app_state = AppState::init(config);

    // If the user wants to manage users, do that and exit
    if args.manage_users {
        match cli_user_management::manage_users(&app_state.get_usersdb_path()) {
            Ok(_) => return,
            Err(e) => {
                tracing::error!("Error managing users: {}", e);
                std::process::exit(1);
            }
        }
    }

    // Start the server
    let addr = SocketAddr::from((app_state.get_server_address(), app_state.get_server_port()));

    let app = get_router(axum::extract::State(app_state.clone()));

    tracing::info!("listening on {}", addr);

    axum_server::bind(addr)
        .handle(Handle::new())
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
}
