use askama::Template;
use auth::AuthState;
use axum::{
    extract::{ConnectInfo, Query, State},
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Response},
};
use common::RateLimiter;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::process::Stdio;
use tokio::process::Command;

use crate::templates::{RouterAdminCommandResult, RouterAdminError};

pub async fn router_admin_command_with_auth(
    State(auth_state): State<AuthState>,
    headers: HeaderMap,
    Query(params): Query<HashMap<String, String>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> Response {
    // Use the rate limiter from AuthState
    let rate_limiter = auth_state.rate_limiter.clone();

    // Call the existing function with the rate limiter
    router_admin_command(
        headers,
        State(rate_limiter),
        Query(params),
        ConnectInfo(addr),
    )
    .await
}

pub async fn router_admin_command(
    _headers: HeaderMap,
    State(rate_limiter): State<RateLimiter>,
    Query(params): Query<HashMap<String, String>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> Response {
    // Rate limit by IP
    let ip = addr.ip().to_string();
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
    let ssh_cmd = match cmd {
        "stats" => vec!["root@satellite.lan", "stats"],
        "reboot" => vec!["root@satellite.lan", "reboot"],
        "poweroff" => vec!["root@satellite.lan", "poweroff"],
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
        .args(&ssh_cmd)
        .args(vec![
            "-o",
            "BatchMode=yes",
            "-o",
            "StrictHostKeyChecking=accept-new",
            "-o",
            "ConnectTimeout=10",
        ])
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
                        String::from_utf8_lossy(&output.stdout).into_owned()
                    } else {
                        format!(
                            "Command failed (exit code: {})\nError: {}",
                            output.status,
                            String::from_utf8_lossy(&output.stderr)
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
