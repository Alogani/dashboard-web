use askama::Template;
use auth::AuthState;
use axum::{
    extract::{ConnectInfo, Query, State},
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Response},
};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::process::Stdio;
use tokio::process::Command;

use crate::templates::{RouterAdminCommandResult, RouterAdminError};

pub async fn router_admin_command(
    State(auth_state): State<AuthState>,
    _headers: HeaderMap,
    Query(params): Query<HashMap<String, String>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> Response {
    // Rate limit by IP
    let ip = addr.ip().to_string();
    let rate_limiter = auth_state.rate_limiter.clone();
    if !rate_limiter.check_rate_limit(&ip) {
        let template = RouterAdminError {
            message: "Too fast. Wait a sec.",
        };
        return match template.render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => {
                tracing::error!("Template error: {}", err);
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        };
    }

    // Map cmd → SSH
    let cmd = params.get("cmd").map(|s| s.as_str()).unwrap_or("");
    match cmd {
        "stats" | "reboot" | "poweroff" => (),
        _ => {
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

    // Execute SSH command asynchronously
    let output = match Command::new("ssh")
        .arg(auth_state.app_config.read().await.router_address.clone())
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
    };

    let template = RouterAdminCommandResult { cmd, output };

    match template.render() {
        Ok(html) => Html(html).into_response(),
        Err(err) => {
            tracing::error!("Template error: {}", err);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
