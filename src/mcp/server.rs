//! HTTP / SSE server exposing the MCP endpoint for local AI agents.

use std::net::SocketAddr;
use std::sync::Arc;

use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::json;
use tokio::sync::mpsc;
use tower_http::cors::CorsLayer;

use super::tools::{call_tool, list_tools};
use super::types::{JsonRpcRequest, JsonRpcResponse};
use crate::backend::Command;

#[derive(Clone)]
pub struct ServerState {
    pub commands: mpsc::UnboundedSender<Command>,
    pub token: Arc<String>,
}

#[derive(Deserialize, Default)]
struct AuthQuery {
    token: Option<String>,
}

fn authenticate(state: &ServerState, headers: &HeaderMap, query: &AuthQuery) -> bool {
    let expected = state.token.trim();
    if expected.is_empty() {
        return false;
    }

    if let Some(auth_header) = headers
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
    {
        if let Some(bearer) = auth_header.strip_prefix("Bearer ").map(str::trim) {
            if bearer == expected {
                return true;
            }
        }
    }

    if let Some(query_token) = &query.token {
        if query_token.trim() == expected {
            return true;
        }
    }

    false
}

pub async fn start_server(
    commands: mpsc::UnboundedSender<Command>,
    port: u16,
    token: String,
    mut shutdown_rx: tokio::sync::watch::Receiver<bool>,
) {
    let state = Arc::new(ServerState {
        commands,
        token: Arc::new(token),
    });

    let cors = CorsLayer::new()
        .allow_origin(tower_http::cors::Any)
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::OPTIONS,
        ])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE]);

    let app = Router::new()
        .route("/mcp", post(handle_jsonrpc))
        .route("/mcp", get(handle_sse))
        .route("/", post(handle_jsonrpc))
        .route("/sse", get(handle_sse))
        .route("/messages", post(handle_jsonrpc))
        .layer(cors)
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    log::info!("Starting ZapFast MCP server at http://{}", addr);

    match tokio::net::TcpListener::bind(addr).await {
        Ok(listener) => {
            let server = axum::serve(listener, app).with_graceful_shutdown(async move {
                while !*shutdown_rx.borrow_and_update() {
                    if shutdown_rx.changed().await.is_err() {
                        break;
                    }
                }
                log::info!("ZapFast MCP server shutting down on http://{}", addr);
            });

            if let Err(err) = server.await {
                log::error!("ZapFast MCP server error: {err}");
            }
        }
        Err(err) => {
            log::error!("Failed to bind MCP server to {addr}: {err}");
        }
    }
}

async fn handle_jsonrpc(
    State(state): State<Arc<ServerState>>,
    Query(query): Query<AuthQuery>,
    headers: HeaderMap,
    Json(request): Json<JsonRpcRequest>,
) -> Response {
    let id = request.id.clone().unwrap_or(serde_json::Value::Null);

    if !authenticate(&state, &headers, &query) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(JsonRpcResponse::error(
                id,
                -32000,
                "Unauthorized: invalid or missing MCP authentication token",
            )),
        )
            .into_response();
    }

    let response = match request.method.as_str() {
        "initialize" => {
            let result = json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {
                        "listChanged": false
                    }
                },
                "serverInfo": {
                    "name": "zapfast-mcp",
                    "version": env!("CARGO_PKG_VERSION")
                }
            });
            JsonRpcResponse::success(id, result)
        }
        "notifications/initialized" | "initialized" => {
            // Client acknowledgment notification, no response required per JSON-RPC notification rules,
            // but return empty 200 OK
            return StatusCode::OK.into_response();
        }
        "ping" => JsonRpcResponse::success(id, json!({})),
        "tools/list" => {
            let tools = list_tools();
            JsonRpcResponse::success(id, json!({ "tools": tools }))
        }
        "tools/call" => {
            let params = request.params.unwrap_or(serde_json::Value::Null);
            let name = params
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            let arguments = params
                .get("arguments")
                .cloned()
                .unwrap_or_else(|| json!({}));

            call_tool(&state.commands, id, name, arguments).await
        }
        _ => JsonRpcResponse::error(id, -32601, format!("Method not found: {}", request.method)),
    };

    Json(response).into_response()
}

async fn handle_sse(
    State(state): State<Arc<ServerState>>,
    Query(query): Query<AuthQuery>,
    headers: HeaderMap,
) -> Response {
    if !authenticate(&state, &headers, &query) {
        return (
            StatusCode::UNAUTHORIZED,
            "Unauthorized: invalid or missing MCP authentication token",
        )
            .into_response();
    }

    let endpoint_url = match &query.token {
        Some(token) if !token.trim().is_empty() => format!("/mcp?token={}", token.trim()),
        _ => "/mcp".to_string(),
    };
    let body = format!("event: endpoint\ndata: {endpoint_url}\n\n");
    Response::builder()
        .header("Content-Type", "text/event-stream")
        .header("Cache-Control", "no-cache")
        .header("Connection", "keep-alive")
        .body(axum::body::Body::from(body))
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
}
