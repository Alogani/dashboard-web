use askama::Template;
use axum::{
    extract::{ConnectInfo, Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Response},
};
use rate_limiter::RateLimiter;
use state::AppState;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::process::Stdio;
use tokio::process::Command;

use crate::templates::{AdminPanels, RouterAdminCommandResult, RouterAdminError};

/// Checks if the request should be rate limited based on IP address
async fn check_rate_limit(rate_limiter: &RateLimiter, ip: &str) -> Option<Response> {
    if !rate_limiter.check_rate_limit(ip) {
        let template = RouterAdminError {
            message: "Too fast. Wait a sec.",
        };
        return match template.render() {
            Ok(html) => Some(Html(html).into_response()),
            Err(err) => {
                tracing::error!("Template error: {}", err);
                Some(StatusCode::INTERNAL_SERVER_ERROR.into_response())
            }
        };
    }
    None
}

async fn execute_command(cmd: &str, ssh_address: &str) -> String {
    if ssh_address.is_empty() {
        // For future implementation of local commands
        return format!("Local command execution not implemented for: {}", cmd);
    }

    // Execute SSH command asynchronously
    match Command::new("ssh")
        .arg(ssh_address)
        .args(vec![
            "-o",
            "BatchMode=yes",
            "-o",
            "StrictHostKeyChecking=accept-new",
            "-o",
            "ConnectTimeout=10",
        ])
        .arg(cmd)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(child) => {
            // Wait for the command to complete and capture output
            match child.wait_with_output().await {
                Ok(output) => {
                    if output.status.success() {
                        tracing::info!("Command '{}' executed successfully", cmd);
                        String::from_utf8_lossy(&output.stdout).into_owned()
                    } else {
                        let error = String::from_utf8_lossy(&output.stderr);
                        tracing::error!(
                            "Command '{}' failed with exit code {}: {}",
                            cmd,
                            output.status,
                            error
                        );
                        format!(
                            "Command failed (exit code: {})\nError: {}",
                            output.status, error
                        )
                    }
                }
                Err(e) => format!("Failed to get command output: {}", e),
            }
        }
        Err(e) => format!("Failed to execute SSH: {}", e),
    }
}

pub async fn router_admin_command(
    State((app_state, rate_limiter)): State<(AppState, RateLimiter)>,
    _headers: HeaderMap,
    path_params: Option<Path<String>>,
    Query(_query_params): Query<HashMap<String, String>>, // Renamed to indicate we're not using it
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> Response {
    // Rate limit by IP
    let ip = addr.ip().to_string();
    if let Some(response) = check_rate_limit(&rate_limiter, &ip).await {
        return response;
    }

    let admin_commands = app_state.get_app_config().get_admin_commands();

    // Get command from path parameter only
    let cmd_key = path_params.map(|Path(cmd_name)| cmd_name);

    // If no command is specified, render the admin panels
    if cmd_key.is_none() {
        let template = AdminPanels {
            panels: admin_commands.get_panels_with_commands(),
        };
        return match template.render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => {
                tracing::error!("Template error: {}", err);
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        };
    }

    // Execute the specified command
    let cmd_key = cmd_key.unwrap();
    let command = match admin_commands.get_commands().get(&cmd_key) {
        Some(cmd) => cmd,
        None => {
            let template = RouterAdminError {
                message: "Invalid command",
            };
            return match template.render() {
                Ok(html) => Html(html).into_response(),
                Err(err) => {
                    tracing::error!("Template error: {}", err);
                    StatusCode::INTERNAL_SERVER_ERROR.into_response()
                }
            };
        }
    };

    // Log the command execution
    tracing::info!("Executing command: {} ({})", &command.name, cmd_key);

    let host = admin_commands.get_hosts().get(&command.host).unwrap();
    let output = execute_command(&command.command, &host).await;

    let template = RouterAdminCommandResult {
        cmd: &command.name,
        output,
    };

    match template.render() {
        Ok(html) => Html(html).into_response(),
        Err(err) => {
            tracing::error!("Template error: {}", err);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
