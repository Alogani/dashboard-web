use std::path::PathBuf;
use std::sync::Arc;

use tracing_subscriber::fmt::Layer;
use tracing_subscriber::fmt::format::{DefaultFields, Format};
use tracing_subscriber::layer::Layered;
use tracing_subscriber::prelude::*;
use tracing_subscriber::reload;
use tracing_subscriber::{EnvFilter, Registry};

use config::LogLevel;

pub type TracingReloadHandler = reload::Handle<
    Layer<Layered<EnvFilter, Registry>, DefaultFields, Format, std::fs::File>,
    Layered<EnvFilter, Registry>,
>;

pub fn setup_logging(
    log_level: &LogLevel,
    log_file: &Option<PathBuf>,
) -> Option<TracingReloadHandler> {
    let registry = tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(log_level.to_string()));

    // Configure logging destination based on config
    match log_file {
        Some(log_path) => {
            // Ensure parent directory exists
            if let Some(parent) = log_path.parent() {
                if !parent.exists() {
                    if let Err(e) = std::fs::create_dir_all(parent) {
                        eprintln!("Failed to create log directory: {}", e);
                    }
                }
            }

            // Log to file
            let file = match std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(log_path)
            {
                Ok(file) => file,
                Err(e) => {
                    eprintln!("Failed to open log file {}: {}", log_path.display(), e);
                    // Fall back to stdout
                    registry.with(tracing_subscriber::fmt::layer()).init();
                    return None;
                }
            };

            let (layer, reload_handle) = reload::Layer::new(
                tracing_subscriber::fmt::layer()
                    .with_writer(file)
                    .with_ansi(false),
            );

            registry.with(layer).init();
            Some(reload_handle)
        }
        None => {
            // Log to stdout if no log file is specified
            registry.with(tracing_subscriber::fmt::layer()).init();
            None
        }
    }
}

pub fn reload_logging(
    tracing_reloader: Arc<Option<TracingReloadHandler>>,
    log_file: &Option<PathBuf>,
) {
    let log_path = match log_file {
        Some(log_path) => log_path,
        None => return,
    };
    let reload_handle = match tracing_reloader.as_ref() {
        Some(handle) => handle,
        None => return,
    };
    let file = match std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)
    {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Failed to open log file {}: {}", log_path.display(), e);
            return;
        }
    };
    if let Err(e) = reload_handle.modify(|filter| {
        *filter = tracing_subscriber::fmt::layer()
            .with_writer(file)
            .with_ansi(false)
    }) {
        eprintln!("Failed to reload logging: {}", e);
    }
}
