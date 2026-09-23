//! MCP data transfer models and JSON-RPC types.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct McpChat {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub unread: u32,
    pub last_activity: i64,
    pub archived: bool,
    pub pinned: bool,
    pub muted: bool,
    pub last_message: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct McpMessage {
    pub id: String,
    pub chat: String,
    pub sender: String,
    pub sender_name: Option<String>,
    pub from_me: bool,
    pub timestamp: i64,
    pub summary: String,
    pub status: String,
    pub quoted: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct McpSendResult {
    pub id: String,
    pub chat: String,
    pub status: String,
}

#[derive(Clone, Debug)]
pub enum McpCommand {
    SearchChats {
        query: Option<String>,
        limit: usize,
        reply: tokio::sync::oneshot::Sender<Result<Vec<McpChat>, String>>,
    },
    GetMessages {
        chat_id: String,
        limit: usize,
        before: Option<(i64, String)>,
        reply: tokio::sync::oneshot::Sender<Result<Vec<McpMessage>, String>>,
    },
    SearchMessages {
        query: String,
        limit: usize,
        reply: tokio::sync::oneshot::Sender<Result<Vec<McpMessage>, String>>,
    },
    SendMessage {
        chat_id: String,
        text: String,
        reply: tokio::sync::oneshot::Sender<Result<McpSendResult, String>>,
    },
    ReplyMessage {
        chat_id: String,
        message_id: String,
        text: String,
        reply: tokio::sync::oneshot::Sender<Result<McpSendResult, String>>,
    },
}

// --- JSON-RPC / MCP Protocol Structs ---

#[derive(Debug, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: Option<String>,
    pub id: Option<serde_json::Value>,
    pub method: String,
    #[serde(default)]
    pub params: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: &'static str,
    pub id: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

impl JsonRpcResponse {
    pub fn success(id: serde_json::Value, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0",
            id,
            result: Some(result),
            error: None,
        }
    }

    pub fn error(id: serde_json::Value, code: i32, message: impl Into<String>) -> Self {
        Self {
            jsonrpc: "2.0",
            id,
            result: None,
            error: Some(JsonRpcError {
                code,
                message: message.into(),
                data: None,
            }),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct McpToolDefinition {
    pub name: &'static str,
    pub description: &'static str,
    #[serde(rename = "inputSchema")]
    pub input_schema: serde_json::Value,
}
