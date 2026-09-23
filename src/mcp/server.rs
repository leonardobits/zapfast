//! HTTP / SSE server exposing the MCP endpoint for local AI agents.

use std::net::SocketAddr;
use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde_json::json;
use tokio::sync::mpsc;
use tower_http::cors::CorsLayer;

use super::tools::{call_tool, list_tools};
use super::types::{JsonRpcRequest, JsonRpcResponse};
use crate::backend::Command;

#[derive(Clone)]
pub struct ServerState {
    pub commands: mpsc::UnboundedSender<Command>,
}

pub async fn start_server(commands: mpsc::UnboundedSender<Command>, port: u16) {
    let state = Arc::new(ServerState { commands });

    let app = Router::new()
        .route("/mcp", post(handle_jsonrpc))
        .route("/mcp", get(handle_sse))
        .route("/", post(handle_jsonrpc))
        .route("/sse", get(handle_sse))
        .route("/messages", post(handle_jsonrpc))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    log::info!("Starting ZapFast MCP server at http://{}", addr);

    match tokio::net::TcpListener::bind(addr).await {
        Ok(listener) => {
            if let Err(err) = axum::serve(listener, app).await {
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
    Json(request): Json<JsonRpcRequest>,
) -> Response {
    let id = request.id.clone().unwrap_or(serde_json::Value::Null);

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
            let name = params.get("name").and_then(|v| v.as_str()).unwrap_or_default();
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

async fn handle_sse() -> Response {
    // SSE endpoint announcement for standard MCP SSE clients
    let body = "event: endpoint\ndata: /mcp\n\n";
    Response::builder()
        .header("Content-Type", "text/event-stream")
        .header("Cache-Control", "no-cache")
        .header("Connection", "keep-alive")
        .body(axum::body::Body::from(body))
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
}
