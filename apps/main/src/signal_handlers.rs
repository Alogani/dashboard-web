use state::AppState;
use tokio::signal::unix::{SignalKind, signal};

pub fn spawn_sighup_watcher(app_state: AppState) {
    let mut sighup = signal(SignalKind::hangup()).expect("Failed to register SIGHUP handler");

    tokio::spawn(async move {
        loop {
            sighup.recv().await;
            println!("Received SIGHUP, reloading configuration...");
            if let Err(e) = app_state.reload_user_config().await {
                eprintln!("Error reloading configuration: {}", e);
            } else {
                println!("Configuration reloaded successfully");
            }
        }
    });
}
