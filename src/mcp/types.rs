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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_message: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct McpReaction {
    pub emoji: String,
    pub sender: String,
    pub from_me: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct McpMedia {
    pub media_type: String,
    pub mime: String,
    pub size_bytes: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,
    pub is_downloaded: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct McpMessage {
    pub id: String,
    pub chat: String,
    pub sender: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sender_name: Option<String>,
    pub from_me: bool,
    pub timestamp: i64,
    pub summary: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media: Option<McpMedia>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub reactions: Vec<McpReaction>,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quoted: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct McpUnreadChatOverview {
    pub chat: McpChat,
    pub messages: Vec<McpMessage>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct McpSendResult {
    pub id: String,
    pub chat: String,
    pub status: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct McpActionResult {
    pub action: String,
    pub chat_id: String,
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_id: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct McpChatContext {
    pub chat_id: String,
    pub name: String,
    pub kind: String,
    pub unread: u32,
    pub transcript: String,
    pub messages_count: usize,
}

#[derive(Debug)]
pub enum McpCommand {
    GetChatContext {
        chat_id: String,
        limit: usize,
        reply: tokio::sync::oneshot::Sender<Result<McpChatContext, String>>,
    },
    PollNewMessages {
        since_timestamp: Option<i64>,
        chat_id: Option<String>,
        limit: usize,
        reply: tokio::sync::oneshot::Sender<Result<Vec<McpMessage>, String>>,
    },
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
    ReactMessage {
        chat_id: String,
        message_id: String,
        emoji: String,
        reply: tokio::sync::oneshot::Sender<Result<bool, String>>,
    },
    MarkRead {
        chat_id: String,
        reply: tokio::sync::oneshot::Sender<Result<bool, String>>,
    },
    GetUnreadOverview {
        chat_limit: usize,
        message_limit: usize,
        reply: tokio::sync::oneshot::Sender<Result<Vec<McpUnreadChatOverview>, String>>,
    },
    DownloadAttachment {
        chat_id: String,
        message_id: String,
        reply: tokio::sync::oneshot::Sender<Result<String, String>>,
    },
    EditMessage {
        chat_id: String,
        message_id: String,
        text: String,
        reply: tokio::sync::oneshot::Sender<Result<McpSendResult, String>>,
    },
    DeleteMessage {
        chat_id: String,
        message_id: String,
        reply: tokio::sync::oneshot::Sender<Result<McpSendResult, String>>,
    },
    BatchEditMessages {
        chat_id: Option<String>,
        edits: Vec<serde_json::Value>,
        reply: tokio::sync::oneshot::Sender<Result<Vec<McpActionResult>, String>>,
    },
    BatchDeleteMessages {
        chat_id: Option<String>,
        message_ids: Vec<String>,
        reply: tokio::sync::oneshot::Sender<Result<Vec<McpActionResult>, String>>,
    },
    BatchActions {
        actions: Vec<serde_json::Value>,
        reply: tokio::sync::oneshot::Sender<Result<Vec<McpActionResult>, String>>,
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
