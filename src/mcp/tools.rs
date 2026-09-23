//! MCP tool definitions and dispatch logic.

use serde_json::json;
use tokio::sync::mpsc;

use super::types::{JsonRpcResponse, McpCommand, McpToolDefinition};
use crate::backend::Command;

pub fn list_tools() -> Vec<McpToolDefinition> {
    vec![
        McpToolDefinition {
            name: "search_chats",
            description: "Search or list WhatsApp chats with basic metadata (unread count, name, last activity).",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Optional search term to filter chat names or phone numbers."
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Maximum number of chats to return (default: 20, max: 100)."
                    }
                }
            }),
        },
        McpToolDefinition {
            name: "get_messages",
            description: "Get recent messages from a specific WhatsApp chat.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "chat_id": {
                        "type": "string",
                        "description": "The WhatsApp JID of the chat (e.g. 551199999999@s.whatsapp.net or 123456@g.us)."
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Maximum number of messages to return (default: 20, max: 100)."
                    }
                },
                "required": ["chat_id"]
            }),
        },
        McpToolDefinition {
            name: "search_messages",
            description: "Search through the local archive of all messages for a keyword or phrase.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "The keyword or phrase to search for."
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Maximum number of messages to return (default: 20, max: 100)."
                    }
                },
                "required": ["query"]
            }),
        },
        McpToolDefinition {
            name: "send_message",
            description: "Send a text message to a WhatsApp chat or phone number.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "chat_id": {
                        "type": "string",
                        "description": "The recipient WhatsApp JID (e.g. 551199999999@s.whatsapp.net or 123456@g.us)."
                    },
                    "text": {
                        "type": "string",
                        "description": "The text content of the message."
                    }
                },
                "required": ["chat_id", "text"]
            }),
        },
        McpToolDefinition {
            name: "reply_message",
            description: "Reply (quote) to a specific message in a WhatsApp chat.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "chat_id": {
                        "type": "string",
                        "description": "The WhatsApp JID of the chat containing the original message."
                    },
                    "message_id": {
                        "type": "string",
                        "description": "The WhatsApp message ID of the message being replied to."
                    },
                    "text": {
                        "type": "string",
                        "description": "The reply message text."
                    }
                },
                "required": ["chat_id", "message_id", "text"]
            }),
        },
    ]
}

pub async fn call_tool(
    commands: &mpsc::UnboundedSender<Command>,
    id: serde_json::Value,
    tool_name: &str,
    arguments: serde_json::Value,
) -> JsonRpcResponse {
    match tool_name {
        "search_chats" => {
            let query = arguments.get("query").and_then(|v| v.as_str()).map(str::to_owned);
            let limit = arguments.get("limit").and_then(|v| v.as_u64()).unwrap_or(20).clamp(1, 100) as usize;

            let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
            if let Err(err) = commands.send(Command::Mcp(McpCommand::SearchChats {
                query,
                limit,
                reply: reply_tx,
            })) {
                return JsonRpcResponse::error(id, -32603, format!("Failed to dispatch command to backend: {err}"));
            }

            match reply_rx.await {
                Ok(Ok(chats)) => {
                    let content = vec![json!({
                        "type": "text",
                        "text": serde_json::to_string_pretty(&chats).unwrap_or_default()
                    })];
                    JsonRpcResponse::success(id, json!({ "content": content }))
                }
                Ok(Err(err)) => JsonRpcResponse::error(id, -32000, err),
                Err(_) => JsonRpcResponse::error(id, -32603, "Backend worker dropped reply channel"),
            }
        }
        "get_messages" => {
            let chat_id = match arguments.get("chat_id").and_then(|v| v.as_str()) {
                Some(s) if !s.trim().is_empty() => s.trim().to_owned(),
                _ => return JsonRpcResponse::error(id, -32602, "Missing or invalid 'chat_id' parameter"),
            };
            let limit = arguments.get("limit").and_then(|v| v.as_u64()).unwrap_or(20).clamp(1, 100) as usize;

            let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
            if let Err(err) = commands.send(Command::Mcp(McpCommand::GetMessages {
                chat_id,
                limit,
                before: None,
                reply: reply_tx,
            })) {
                return JsonRpcResponse::error(id, -32603, format!("Failed to dispatch command to backend: {err}"));
            }

            match reply_rx.await {
                Ok(Ok(messages)) => {
                    let content = vec![json!({
                        "type": "text",
                        "text": serde_json::to_string_pretty(&messages).unwrap_or_default()
                    })];
                    JsonRpcResponse::success(id, json!({ "content": content }))
                }
                Ok(Err(err)) => JsonRpcResponse::error(id, -32000, err),
                Err(_) => JsonRpcResponse::error(id, -32603, "Backend worker dropped reply channel"),
            }
        }
        "search_messages" => {
            let query = match arguments.get("query").and_then(|v| v.as_str()) {
                Some(s) if !s.trim().is_empty() => s.trim().to_owned(),
                _ => return JsonRpcResponse::error(id, -32602, "Missing or invalid 'query' parameter"),
            };
            let limit = arguments.get("limit").and_then(|v| v.as_u64()).unwrap_or(20).clamp(1, 100) as usize;

            let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
            if let Err(err) = commands.send(Command::Mcp(McpCommand::SearchMessages {
                query,
                limit,
                reply: reply_tx,
            })) {
                return JsonRpcResponse::error(id, -32603, format!("Failed to dispatch command to backend: {err}"));
            }

            match reply_rx.await {
                Ok(Ok(messages)) => {
                    let content = vec![json!({
                        "type": "text",
                        "text": serde_json::to_string_pretty(&messages).unwrap_or_default()
                    })];
                    JsonRpcResponse::success(id, json!({ "content": content }))
                }
                Ok(Err(err)) => JsonRpcResponse::error(id, -32000, err),
                Err(_) => JsonRpcResponse::error(id, -32603, "Backend worker dropped reply channel"),
            }
        }
        "send_message" => {
            let chat_id = match arguments.get("chat_id").and_then(|v| v.as_str()) {
                Some(s) if !s.trim().is_empty() => s.trim().to_owned(),
                _ => return JsonRpcResponse::error(id, -32602, "Missing or invalid 'chat_id' parameter"),
            };
            let text = match arguments.get("text").and_then(|v| v.as_str()) {
                Some(s) if !s.is_empty() => s.to_owned(),
                _ => return JsonRpcResponse::error(id, -32602, "Missing or invalid 'text' parameter"),
            };

            let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
            if let Err(err) = commands.send(Command::Mcp(McpCommand::SendMessage {
                chat_id,
                text,
                reply: reply_tx,
            })) {
                return JsonRpcResponse::error(id, -32603, format!("Failed to dispatch command to backend: {err}"));
            }

            match reply_rx.await {
                Ok(Ok(res)) => {
                    let content = vec![json!({
                        "type": "text",
                        "text": serde_json::to_string_pretty(&res).unwrap_or_default()
                    })];
                    JsonRpcResponse::success(id, json!({ "content": content }))
                }
                Ok(Err(err)) => JsonRpcResponse::error(id, -32000, err),
                Err(_) => JsonRpcResponse::error(id, -32603, "Backend worker dropped reply channel"),
            }
        }
        "reply_message" => {
            let chat_id = match arguments.get("chat_id").and_then(|v| v.as_str()) {
                Some(s) if !s.trim().is_empty() => s.trim().to_owned(),
                _ => return JsonRpcResponse::error(id, -32602, "Missing or invalid 'chat_id' parameter"),
            };
            let message_id = match arguments.get("message_id").and_then(|v| v.as_str()) {
                Some(s) if !s.trim().is_empty() => s.trim().to_owned(),
                _ => return JsonRpcResponse::error(id, -32602, "Missing or invalid 'message_id' parameter"),
            };
            let text = match arguments.get("text").and_then(|v| v.as_str()) {
                Some(s) if !s.is_empty() => s.to_owned(),
                _ => return JsonRpcResponse::error(id, -32602, "Missing or invalid 'text' parameter"),
            };

            let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
            if let Err(err) = commands.send(Command::Mcp(McpCommand::ReplyMessage {
                chat_id,
                message_id,
                text,
                reply: reply_tx,
            })) {
                return JsonRpcResponse::error(id, -32603, format!("Failed to dispatch command to backend: {err}"));
            }

            match reply_rx.await {
                Ok(Ok(res)) => {
                    let content = vec![json!({
                        "type": "text",
                        "text": serde_json::to_string_pretty(&res).unwrap_or_default()
                    })];
                    JsonRpcResponse::success(id, json!({ "content": content }))
                }
                Ok(Err(err)) => JsonRpcResponse::error(id, -32000, err),
                Err(_) => JsonRpcResponse::error(id, -32603, "Backend worker dropped reply channel"),
            }
        }
        unknown => JsonRpcResponse::error(id, -32601, format!("Unknown tool: {unknown}")),
    }
}
