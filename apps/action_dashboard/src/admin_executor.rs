use askama::Template;
use axum::{
    extract::{ConnectInfo, Path, State},
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Response},
};
use limiters_middleware::RateLimiter;
use state::AppState;
use std::net::SocketAddr;
use std::process::Stdio;
use std::time::Duration;
use tokio::process::Command;
use utils::http_helpers::{get_client_ip, is_same_origin_request};
use utils::with_nocache;

use crate::templates::{ActionResult, ExecutionError};

const COMMAND_TIMEOUT: Duration = Duration::from_secs(120);

fn render_error(status: StatusCode, message: &str) -> Response {
    let template = ExecutionError { message };
    match template.render() {
        Ok(html) => with_nocache!((status, Html(html))),
        Err(err) => {
            tracing::error!("Template error: {}", err);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn execute_command(cmd: &str, ssh_address: &str) -> String {
    let mut command = if ssh_address.is_empty() {
        let mut command = Command::new("sh");
        command.args(["-c", cmd]);
        command
    } else {
        let mut command = Command::new("ssh");
        command
            .args([
                "-o",
                "BatchMode=yes",
                "-o",
                "StrictHostKeyChecking=accept-new",
                "-o",
                "ConnectTimeout=10",
                "--",
            ])
            .arg(ssh_address)
            .arg(cmd);
        command
    };
    let child_process = command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn();

    match child_process {
        Ok(child) => {
            // Wait for the command to complete and capture output
            match tokio::time::timeout(COMMAND_TIMEOUT, child.wait_with_output()).await {
                Ok(Ok(output)) => {
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
                Ok(Err(e)) => format!("Failed to get command output: {}", e),
                Err(_) => {
                    tracing::error!("Command '{}' timed out", cmd);
                    format!(
                        "Command timed out after {} seconds",
                        COMMAND_TIMEOUT.as_secs()
                    )
                }
            }
        }
        Err(e) => format!("Failed to execute command: {}", e),
    }
}

pub async fn execute_admin_action(
    State((app_state, rate_limiter)): State<(AppState, RateLimiter)>,
    headers: HeaderMap,
    Path(cmd_key): Path<String>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> Response {
    // Commands have side effects: refuse requests forged by other sites (CSRF)
    if !is_same_origin_request(&headers) {
        tracing::warn!("Rejected cross-origin request to execute '{}'", cmd_key);
        return render_error(StatusCode::FORBIDDEN, "Cross-origin request rejected");
    }

    // Rate limit by IP
    let ip = get_client_ip(&headers, &addr);
    if !rate_limiter.check_limit(&ip) {
        return render_error(StatusCode::TOO_MANY_REQUESTS, "Too fast. Wait a sec.");
    }

    let admin_cmds = app_state.get_app_config().get_admin_commands();
    let cmd_info = match admin_cmds.get_action_by_url(&cmd_key) {
        Some(cmd) => cmd,
        None => return render_error(StatusCode::NOT_FOUND, "Invalid command"),
    };

    // Log the command execution
    tracing::info!(
        "Executing command: {} ({}) requested from IP {}",
        &cmd_info.command,
        &cmd_info.url_name,
        ip
    );

    let host = match admin_cmds.hosts.get(&cmd_info.host) {
        Some(host) => host,
        None => return render_error(StatusCode::INTERNAL_SERVER_ERROR, "Invalid host"),
    };
    let output = execute_command(&cmd_info.command, &host.address).await;

    let template = ActionResult {
        action_name: &cmd_info.pretty_name,
        output,
    };

    match template.render() {
        Ok(html) => with_nocache!(Html(html)),
        Err(err) => {
            tracing::error!("Template error: {}", err);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
