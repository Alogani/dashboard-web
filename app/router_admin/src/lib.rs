use askama::Template;
use auth::AuthState;
use axum::{
    extract::{ConnectInfo, Query, State},
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};
use common::RateLimiter;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::process::{Command, Stdio};

mod templates;

use crate::templates::{RouterAdminCommandResult, RouterAdminError, RouterAdminLanding};

pub fn router() -> Router<AuthState> {
    Router::new()
        .route(
            "/",
            get(|| async {
                let template = RouterAdminLanding;
                match template.render() {
                    Ok(html) => Html(html).into_response(),
                    Err(err) => {
                        tracing::error!("Template error: {}", err);
                        StatusCode::INTERNAL_SERVER_ERROR.into_response()
                    }
                }
            }),
        )
        .route("/command", get(router_admin_command_with_auth))
}

async fn router_admin_command_with_auth(
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

async fn router_admin_command(
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

    let output = Command::new("ssh")
        .args(&ssh_cmd)
        .stdin(Stdio::null())
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
        .unwrap_or_else(|e| format!("SSH failed: {}", e));

    let template = RouterAdminCommandResult { cmd, output };

    match template.render() {
        Ok(html) => Html(html).into_response(),
        Err(err) => {
            tracing::error!("Template error: {}", err);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
