use askama::Template;
use axum::{
    extract::{ConnectInfo, Query, State},
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Response},
};
use state::AppState;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::process::Stdio;
use tokio::process::Command;

use crate::templates::{AdminPanels, RouterAdminCommandResult, RouterAdminError};

/// Checks if the request should be rate limited based on IP address
async fn check_rate_limit(app_state: &AppState, ip: &str) -> Option<Response> {
    let rate_limiter = app_state.get_rate_limiter().clone();
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
    State(app_state): State<AppState>,
    _headers: HeaderMap,
    Query(params): Query<HashMap<String, String>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> Response {
    // Rate limit by IP
    let ip = addr.ip().to_string();
    if let Some(response) = check_rate_limit(&app_state, &ip).await {
        return response;
    }

    let admin_commands = app_state.get_app_config().get_admin_commands();

    // If no command is specified, render the admin panels
    if !params.contains_key("cmd") {
        let panels: Vec<(&str, Vec<&str>)> = admin_commands
            .get_panels()
            .iter()
            .map(|(panel_key, _)| {
                let commands: Vec<&str> = admin_commands
                    .get_commands()
                    .iter()
                    .filter(|(_, cmd)| cmd.panel == *panel_key)
                    .map(|(cmd_key, _)| cmd_key.as_str())
                    .collect();
                (panel_key.as_str(), commands)
            })
            .collect();

        let template = AdminPanels { panels };
        return match template.render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => {
                tracing::error!("Template error: {}", err);
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        };
    }

    // Execute the specified command
    let cmd_key = params.get("cmd").unwrap();
    let command = match admin_commands.get_commands().get(cmd_key) {
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

    let host = command.host.clone();
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
