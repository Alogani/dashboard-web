use crate::AppState;
use tokio::signal::unix::{SignalKind, signal};

pub fn spawn_sighup_watcher(app_state: AppState) {
    let mut sighup = signal(SignalKind::hangup()).expect("Failed to register SIGHUP handler");

    tokio::spawn(async move {
        loop {
            sighup.recv().await;
            tracing::info!("Received SIGHUP, reloading configuration...");
            if let Err(e) = app_state.reload_all_config().await {
                tracing::error!("Error reloading configuration: {}", e);
            } else {
                tracing::info!("Configuration reloaded successfully");
            }
        }
    });
}
