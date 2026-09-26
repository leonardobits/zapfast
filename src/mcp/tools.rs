//! MCP tool definitions and dispatch logic.

use serde_json::json;
use tokio::sync::mpsc;

use super::types::{JsonRpcResponse, McpCommand, McpToolDefinition};
use crate::backend::Command;

pub fn list_tools() -> Vec<McpToolDefinition> {
    vec![
        McpToolDefinition {
            name: "get_chat_context",
            description: "Returns a chronological conversation transcript with speaker names, quotes, timestamps, and media file paths.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "chat_id": {
                        "type": "string",
                        "description": "The WhatsApp JID of the chat or group (e.g. 551199999999@s.whatsapp.net or 123456@g.us)."
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Number of recent messages to include in the context (default: 30, max: 100)."
                    }
                },
                "required": ["chat_id"]
            }),
        },
        McpToolDefinition {
            name: "poll_new_messages",
            description: "Returns new messages received after since_timestamp, optionally filtered by chat_id.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "since_timestamp": {
                        "type": "integer",
                        "description": "Unix timestamp in seconds. Returns messages received after this timestamp."
                    },
                    "chat_id": {
                        "type": "string",
                        "description": "Optional chat JID to filter messages from only a specific contact or group."
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Maximum number of messages to return (default: 20, max: 50)."
                    }
                }
            }),
        },
        McpToolDefinition {
            name: "get_unread_overview",
            description: "Returns all chats with unread messages and their latest messages.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "chat_limit": {
                        "type": "integer",
                        "description": "Maximum number of unread chats to return (default: 20, max: 50)."
                    },
                    "message_limit": {
                        "type": "integer",
                        "description": "Maximum number of recent messages to return per chat (default: 5, max: 20)."
                    }
                }
            }),
        },
        McpToolDefinition {
            name: "search_chats",
            description: "Search or list WhatsApp chats with metadata (unread count, name, kind, last activity).",
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
            description: "Get recent messages from a specific WhatsApp chat, including text, media metadata, reactions, and quote information.",
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
        McpToolDefinition {
            name: "react_message",
            description: "React to a WhatsApp message with an emoji (e.g. '👍', '❤️', '🔥', '😂', '🎉') or empty string '' to remove a reaction.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "chat_id": {
                        "type": "string",
                        "description": "The WhatsApp JID of the chat containing the message."
                    },
                    "message_id": {
                        "type": "string",
                        "description": "The WhatsApp message ID to react to."
                    },
                    "emoji": {
                        "type": "string",
                        "description": "The emoji to react with (e.g. '👍', '❤️'), or empty '' to remove."
                    }
                },
                "required": ["chat_id", "message_id", "emoji"]
            }),
        },
        McpToolDefinition {
            name: "mark_as_read",
            description: "Mark all messages in a WhatsApp chat as read.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "chat_id": {
                        "type": "string",
                        "description": "The WhatsApp JID of the chat to mark as read."
                    }
                },
                "required": ["chat_id"]
            }),
        },
        McpToolDefinition {
            name: "download_attachment",
            description: "Download a media attachment (photo, audio, document, video, sticker) from a message and return the local absolute file path for instant reading/processing.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "chat_id": {
                        "type": "string",
                        "description": "The WhatsApp JID of the chat containing the message."
                    },
                    "message_id": {
                        "type": "string",
                        "description": "The WhatsApp message ID containing the attachment."
                    }
                },
                "required": ["chat_id", "message_id"]
            }),
        },
        McpToolDefinition {
            name: "edit_message",
            description: "Edit an existing text message previously sent by you in real-time.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "chat_id": {
                        "type": "string",
                        "description": "The WhatsApp JID or name of the chat containing the message."
                    },
                    "message_id": {
                        "type": "string",
                        "description": "The WhatsApp message ID of the message to edit."
                    },
                    "text": {
                        "type": "string",
                        "description": "The new replacement text for the message."
                    }
                },
                "required": ["chat_id", "message_id", "text"]
            }),
        },
        McpToolDefinition {
            name: "delete_message",
            description: "Revoke/delete a message for everyone in a chat (Apagar para todos).",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "chat_id": {
                        "type": "string",
                        "description": "The WhatsApp JID or name of the chat containing the message."
                    },
                    "message_id": {
                        "type": "string",
                        "description": "The WhatsApp message ID to revoke/delete for everyone."
                    }
                },
                "required": ["chat_id", "message_id"]
            }),
        },
        McpToolDefinition {
            name: "batch_edit_messages",
            description: "Edit multiple messages in a single fast call.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "chat_id": {
                        "type": "string",
                        "description": "Optional default WhatsApp JID or name of the chat."
                    },
                    "edits": {
                        "type": "array",
                        "description": "List of edits to apply. Each item must have 'message_id' and 'text', and optional 'chat_id'.",
                        "items": {
                            "type": "object",
                            "properties": {
                                "chat_id": { "type": "string" },
                                "message_id": { "type": "string" },
                                "text": { "type": "string" }
                            },
                            "required": ["message_id", "text"]
                        }
                    }
                },
                "required": ["edits"]
            }),
        },
        McpToolDefinition {
            name: "batch_delete_messages",
            description: "Revoke/delete multiple messages for everyone in a single fast call (Apagar mensagens para todos).",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "chat_id": {
                        "type": "string",
                        "description": "The WhatsApp JID or name of the chat."
                    },
                    "message_ids": {
                        "type": "array",
                        "description": "List of WhatsApp message IDs to revoke/delete for everyone.",
                        "items": { "type": "string" }
                    }
                },
                "required": ["chat_id", "message_ids"]
            }),
        },
        McpToolDefinition {
            name: "batch_actions",
            description: "Execute multiple actions (send, reply, quote, react, mark_read, edit, update, delete, revoke) in a single roundtrip for extreme speed.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "actions": {
                        "type": "array",
                        "description": "List of action objects. Each must have 'action' ('send'|'reply'|'quote'|'react'|'mark_read'|'edit'|'update'|'delete'|'revoke') and 'chat_id', plus required fields for that action type.",
                        "items": {
                            "type": "object",
                            "properties": {
                                "action": {
                                    "type": "string",
                                    "enum": ["send", "reply", "quote", "react", "mark_read", "edit", "update", "delete", "revoke"]
                                },
                                "chat_id": { "type": "string" },
                                "text": { "type": "string" },
                                "message_id": { "type": "string" },
                                "emoji": { "type": "string" }
                            },
                            "required": ["action", "chat_id"]
                        }
                    }
                },
                "required": ["actions"]
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
        "get_chat_context" => {
            let chat_id = match arguments.get("chat_id").and_then(|v| v.as_str()) {
                Some(s) if !s.trim().is_empty() => s.trim().to_owned(),
                _ => {
                    return JsonRpcResponse::error(
                        id,
                        -32602,
                        "Missing or invalid 'chat_id' parameter",
                    );
                }
            };
            let limit = arguments
                .get("limit")
                .and_then(|v| v.as_u64())
                .unwrap_or(30)
                .clamp(1, 100) as usize;

            let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
            if let Err(err) = commands.send(Command::Mcp(McpCommand::GetChatContext {
                chat_id,
                limit,
                reply: reply_tx,
            })) {
                return JsonRpcResponse::error(
                    id,
                    -32603,
                    format!("Failed to dispatch command to backend: {err}"),
                );
            }

            match reply_rx.await {
                Ok(Ok(context)) => {
                    let content = vec![json!({
                        "type": "text",
                        "text": context.transcript
                    })];
                    JsonRpcResponse::success(id, json!({ "content": content }))
                }
                Ok(Err(err)) => JsonRpcResponse::error(id, -32000, err),
                Err(_) => {
                    JsonRpcResponse::error(id, -32603, "Backend worker dropped reply channel")
                }
            }
        }
        "poll_new_messages" => {
            let since_timestamp = arguments.get("since_timestamp").and_then(|v| v.as_i64());
            let chat_id = arguments
                .get("chat_id")
                .and_then(|v| v.as_str())
                .map(str::to_owned);
            let limit = arguments
                .get("limit")
                .and_then(|v| v.as_u64())
                .unwrap_or(20)
                .clamp(1, 50) as usize;

            let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
            if let Err(err) = commands.send(Command::Mcp(McpCommand::PollNewMessages {
                since_timestamp,
                chat_id,
                limit,
                reply: reply_tx,
            })) {
                return JsonRpcResponse::error(
                    id,
                    -32603,
                    format!("Failed to dispatch command to backend: {err}"),
                );
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
                Err(_) => {
                    JsonRpcResponse::error(id, -32603, "Backend worker dropped reply channel")
                }
            }
        }
        "get_unread_overview" => {
            let chat_limit = arguments
                .get("chat_limit")
                .and_then(|v| v.as_u64())
                .unwrap_or(20)
                .clamp(1, 50) as usize;
            let message_limit = arguments
                .get("message_limit")
                .and_then(|v| v.as_u64())
                .unwrap_or(5)
                .clamp(1, 20) as usize;

            let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
            if let Err(err) = commands.send(Command::Mcp(McpCommand::GetUnreadOverview {
                chat_limit,
                message_limit,
                reply: reply_tx,
            })) {
                return JsonRpcResponse::error(
                    id,
                    -32603,
                    format!("Failed to dispatch command to backend: {err}"),
                );
            }

            match reply_rx.await {
                Ok(Ok(overview)) => {
                    let content = vec![json!({
                        "type": "text",
                        "text": serde_json::to_string_pretty(&overview).unwrap_or_default()
                    })];
                    JsonRpcResponse::success(id, json!({ "content": content }))
                }
                Ok(Err(err)) => JsonRpcResponse::error(id, -32000, err),
                Err(_) => {
                    JsonRpcResponse::error(id, -32603, "Backend worker dropped reply channel")
                }
            }
        }
        "search_chats" => {
            let query = arguments
                .get("query")
                .and_then(|v| v.as_str())
                .map(str::to_owned);
            let limit = arguments
                .get("limit")
                .and_then(|v| v.as_u64())
                .unwrap_or(20)
                .clamp(1, 100) as usize;

            let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
            if let Err(err) = commands.send(Command::Mcp(McpCommand::SearchChats {
                query,
                limit,
                reply: reply_tx,
            })) {
                return JsonRpcResponse::error(
                    id,
                    -32603,
                    format!("Failed to dispatch command to backend: {err}"),
                );
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
                Err(_) => {
                    JsonRpcResponse::error(id, -32603, "Backend worker dropped reply channel")
                }
            }
        }
        "get_messages" => {
            let chat_id = match arguments.get("chat_id").and_then(|v| v.as_str()) {
                Some(s) if !s.trim().is_empty() => s.trim().to_owned(),
                _ => {
                    return JsonRpcResponse::error(
                        id,
                        -32602,
                        "Missing or invalid 'chat_id' parameter",
                    );
                }
            };
            let limit = arguments
                .get("limit")
                .and_then(|v| v.as_u64())
                .unwrap_or(20)
                .clamp(1, 100) as usize;

            let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
            if let Err(err) = commands.send(Command::Mcp(McpCommand::GetMessages {
                chat_id,
                limit,
                before: None,
                reply: reply_tx,
            })) {
                return JsonRpcResponse::error(
                    id,
                    -32603,
                    format!("Failed to dispatch command to backend: {err}"),
                );
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
                Err(_) => {
                    JsonRpcResponse::error(id, -32603, "Backend worker dropped reply channel")
                }
            }
        }
        "search_messages" => {
            let query = match arguments.get("query").and_then(|v| v.as_str()) {
                Some(s) if !s.trim().is_empty() => s.trim().to_owned(),
                _ => {
                    return JsonRpcResponse::error(
                        id,
                        -32602,
                        "Missing or invalid 'query' parameter",
                    );
                }
            };
            let limit = arguments
                .get("limit")
                .and_then(|v| v.as_u64())
                .unwrap_or(20)
                .clamp(1, 100) as usize;

            let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
            if let Err(err) = commands.send(Command::Mcp(McpCommand::SearchMessages {
                query,
                limit,
                reply: reply_tx,
            })) {
                return JsonRpcResponse::error(
                    id,
                    -32603,
                    format!("Failed to dispatch command to backend: {err}"),
                );
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
                Err(_) => {
                    JsonRpcResponse::error(id, -32603, "Backend worker dropped reply channel")
                }
            }
        }
        "send_message" => {
            let chat_id = match arguments.get("chat_id").and_then(|v| v.as_str()) {
                Some(s) if !s.trim().is_empty() => s.trim().to_owned(),
                _ => {
                    return JsonRpcResponse::error(
                        id,
                        -32602,
                        "Missing or invalid 'chat_id' parameter",
                    );
                }
            };
            let text = match arguments.get("text").and_then(|v| v.as_str()) {
                Some(s) if !s.is_empty() => s.to_owned(),
                _ => {
                    return JsonRpcResponse::error(
                        id,
                        -32602,
                        "Missing or invalid 'text' parameter",
                    );
                }
            };

            let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
            if let Err(err) = commands.send(Command::Mcp(McpCommand::SendMessage {
                chat_id,
                text,
                reply: reply_tx,
            })) {
                return JsonRpcResponse::error(
                    id,
                    -32603,
                    format!("Failed to dispatch command to backend: {err}"),
                );
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
                Err(_) => {
                    JsonRpcResponse::error(id, -32603, "Backend worker dropped reply channel")
                }
            }
        }
        "reply_message" | "quote_message" => {
            let chat_id = match arguments.get("chat_id").and_then(|v| v.as_str()) {
                Some(s) if !s.trim().is_empty() => s.trim().to_owned(),
                _ => {
                    return JsonRpcResponse::error(
                        id,
                        -32602,
                        "Missing or invalid 'chat_id' parameter",
                    );
                }
            };
            let message_id = match arguments.get("message_id").and_then(|v| v.as_str()) {
                Some(s) if !s.trim().is_empty() => s.trim().to_owned(),
                _ => {
                    return JsonRpcResponse::error(
                        id,
                        -32602,
                        "Missing or invalid 'message_id' parameter",
                    );
                }
            };
            let text = match arguments.get("text").and_then(|v| v.as_str()) {
                Some(s) if !s.is_empty() => s.to_owned(),
                _ => {
                    return JsonRpcResponse::error(
                        id,
                        -32602,
                        "Missing or invalid 'text' parameter",
                    );
                }
            };

            let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
            if let Err(err) = commands.send(Command::Mcp(McpCommand::ReplyMessage {
                chat_id,
                message_id,
                text,
                reply: reply_tx,
            })) {
                return JsonRpcResponse::error(
                    id,
                    -32603,
                    format!("Failed to dispatch command to backend: {err}"),
                );
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
                Err(_) => {
                    JsonRpcResponse::error(id, -32603, "Backend worker dropped reply channel")
                }
            }
        }
        "edit_message" | "update_message" => {
            let chat_id = match arguments.get("chat_id").and_then(|v| v.as_str()) {
                Some(s) if !s.trim().is_empty() => s.trim().to_owned(),
                _ => {
                    return JsonRpcResponse::error(
                        id,
                        -32602,
                        "Missing or invalid 'chat_id' parameter",
                    );
                }
            };
            let message_id = match arguments.get("message_id").and_then(|v| v.as_str()) {
                Some(s) if !s.trim().is_empty() => s.trim().to_owned(),
                _ => {
                    return JsonRpcResponse::error(
                        id,
                        -32602,
                        "Missing or invalid 'message_id' parameter",
                    );
                }
            };
            let text = match arguments.get("text").and_then(|v| v.as_str()) {
                Some(s) if !s.is_empty() => s.to_owned(),
                _ => {
                    return JsonRpcResponse::error(
                        id,
                        -32602,
                        "Missing or invalid 'text' parameter",
                    );
                }
            };

            let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
            if let Err(err) = commands.send(Command::Mcp(McpCommand::EditMessage {
                chat_id,
                message_id,
                text,
                reply: reply_tx,
            })) {
                return JsonRpcResponse::error(
                    id,
                    -32603,
                    format!("Failed to dispatch command to backend: {err}"),
                );
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
                Err(_) => {
                    JsonRpcResponse::error(id, -32603, "Backend worker dropped reply channel")
                }
            }
        }
        "delete_message" | "revoke_message" => {
            let chat_id = match arguments.get("chat_id").and_then(|v| v.as_str()) {
                Some(s) if !s.trim().is_empty() => s.trim().to_owned(),
                _ => {
                    return JsonRpcResponse::error(
                        id,
                        -32602,
                        "Missing or invalid 'chat_id' parameter",
                    );
                }
            };
            let message_id = match arguments.get("message_id").and_then(|v| v.as_str()) {
                Some(s) if !s.trim().is_empty() => s.trim().to_owned(),
                _ => {
                    return JsonRpcResponse::error(
                        id,
                        -32602,
                        "Missing or invalid 'message_id' parameter",
                    );
                }
            };

            let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
            if let Err(err) = commands.send(Command::Mcp(McpCommand::DeleteMessage {
                chat_id,
                message_id,
                reply: reply_tx,
            })) {
                return JsonRpcResponse::error(
                    id,
                    -32603,
                    format!("Failed to dispatch command to backend: {err}"),
                );
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
                Err(_) => {
                    JsonRpcResponse::error(id, -32603, "Backend worker dropped reply channel")
                }
            }
        }
        "batch_edit_messages" => {
            let chat_id = arguments
                .get("chat_id")
                .and_then(|v| v.as_str())
                .map(str::to_owned);
            let edits = match arguments.get("edits").and_then(|v| v.as_array()) {
                Some(arr) => arr.clone(),
                _ => {
                    return JsonRpcResponse::error(
                        id,
                        -32602,
                        "Missing or invalid 'edits' array parameter",
                    );
                }
            };

            let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
            if let Err(err) = commands.send(Command::Mcp(McpCommand::BatchEditMessages {
                chat_id,
                edits,
                reply: reply_tx,
            })) {
                return JsonRpcResponse::error(
                    id,
                    -32603,
                    format!("Failed to dispatch command to backend: {err}"),
                );
            }

            match reply_rx.await {
                Ok(Ok(results)) => {
                    let content = vec![json!({
                        "type": "text",
                        "text": serde_json::to_string_pretty(&results).unwrap_or_default()
                    })];
                    JsonRpcResponse::success(id, json!({ "content": content }))
                }
                Ok(Err(err)) => JsonRpcResponse::error(id, -32000, err),
                Err(_) => {
                    JsonRpcResponse::error(id, -32603, "Backend worker dropped reply channel")
                }
            }
        }
        "batch_delete_messages" => {
            let chat_id = arguments
                .get("chat_id")
                .and_then(|v| v.as_str())
                .map(str::to_owned);
            let message_ids: Vec<String> =
                match arguments.get("message_ids").and_then(|v| v.as_array()) {
                    Some(arr) => arr
                        .iter()
                        .filter_map(|v| v.as_str().map(str::to_owned))
                        .collect(),
                    _ => {
                        return JsonRpcResponse::error(
                            id,
                            -32602,
                            "Missing or invalid 'message_ids' array parameter",
                        );
                    }
                };

            let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
            if let Err(err) = commands.send(Command::Mcp(McpCommand::BatchDeleteMessages {
                chat_id,
                message_ids,
                reply: reply_tx,
            })) {
                return JsonRpcResponse::error(
                    id,
                    -32603,
                    format!("Failed to dispatch command to backend: {err}"),
                );
            }

            match reply_rx.await {
                Ok(Ok(results)) => {
                    let content = vec![json!({
                        "type": "text",
                        "text": serde_json::to_string_pretty(&results).unwrap_or_default()
                    })];
                    JsonRpcResponse::success(id, json!({ "content": content }))
                }
                Ok(Err(err)) => JsonRpcResponse::error(id, -32000, err),
                Err(_) => {
                    JsonRpcResponse::error(id, -32603, "Backend worker dropped reply channel")
                }
            }
        }
        "react_message" => {
            let chat_id = match arguments.get("chat_id").and_then(|v| v.as_str()) {
                Some(s) if !s.trim().is_empty() => s.trim().to_owned(),
                _ => {
                    return JsonRpcResponse::error(
                        id,
                        -32602,
                        "Missing or invalid 'chat_id' parameter",
                    );
                }
            };
            let message_id = match arguments.get("message_id").and_then(|v| v.as_str()) {
                Some(s) if !s.trim().is_empty() => s.trim().to_owned(),
                _ => {
                    return JsonRpcResponse::error(
                        id,
                        -32602,
                        "Missing or invalid 'message_id' parameter",
                    );
                }
            };
            let emoji = arguments
                .get("emoji")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_owned();

            let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
            if let Err(err) = commands.send(Command::Mcp(McpCommand::ReactMessage {
                chat_id,
                message_id,
                emoji,
                reply: reply_tx,
            })) {
                return JsonRpcResponse::error(
                    id,
                    -32603,
                    format!("Failed to dispatch command to backend: {err}"),
                );
            }

            match reply_rx.await {
                Ok(Ok(success)) => {
                    let content = vec![json!({
                        "type": "text",
                        "text": json!({ "success": success }).to_string()
                    })];
                    JsonRpcResponse::success(id, json!({ "content": content }))
                }
                Ok(Err(err)) => JsonRpcResponse::error(id, -32000, err),
                Err(_) => {
                    JsonRpcResponse::error(id, -32603, "Backend worker dropped reply channel")
                }
            }
        }
        "mark_as_read" => {
            let chat_id = match arguments.get("chat_id").and_then(|v| v.as_str()) {
                Some(s) if !s.trim().is_empty() => s.trim().to_owned(),
                _ => {
                    return JsonRpcResponse::error(
                        id,
                        -32602,
                        "Missing or invalid 'chat_id' parameter",
                    );
                }
            };

            let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
            if let Err(err) = commands.send(Command::Mcp(McpCommand::MarkRead {
                chat_id,
                reply: reply_tx,
            })) {
                return JsonRpcResponse::error(
                    id,
                    -32603,
                    format!("Failed to dispatch command to backend: {err}"),
                );
            }

            match reply_rx.await {
                Ok(Ok(success)) => {
                    let content = vec![json!({
                        "type": "text",
                        "text": json!({ "success": success }).to_string()
                    })];
                    JsonRpcResponse::success(id, json!({ "content": content }))
                }
                Ok(Err(err)) => JsonRpcResponse::error(id, -32000, err),
                Err(_) => {
                    JsonRpcResponse::error(id, -32603, "Backend worker dropped reply channel")
                }
            }
        }
        "download_attachment" => {
            let chat_id = match arguments.get("chat_id").and_then(|v| v.as_str()) {
                Some(s) if !s.trim().is_empty() => s.trim().to_owned(),
                _ => {
                    return JsonRpcResponse::error(
                        id,
                        -32602,
                        "Missing or invalid 'chat_id' parameter",
                    );
                }
            };
            let message_id = match arguments.get("message_id").and_then(|v| v.as_str()) {
                Some(s) if !s.trim().is_empty() => s.trim().to_owned(),
                _ => {
                    return JsonRpcResponse::error(
                        id,
                        -32602,
                        "Missing or invalid 'message_id' parameter",
                    );
                }
            };

            let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
            if let Err(err) = commands.send(Command::Mcp(McpCommand::DownloadAttachment {
                chat_id,
                message_id,
                reply: reply_tx,
            })) {
                return JsonRpcResponse::error(
                    id,
                    -32603,
                    format!("Failed to dispatch command to backend: {err}"),
                );
            }

            match reply_rx.await {
                Ok(Ok(file_path)) => {
                    let content = vec![json!({
                        "type": "text",
                        "text": json!({ "file_path": file_path, "status": "downloaded" }).to_string()
                    })];
                    JsonRpcResponse::success(id, json!({ "content": content }))
                }
                Ok(Err(err)) => JsonRpcResponse::error(id, -32000, err),
                Err(_) => {
                    JsonRpcResponse::error(id, -32603, "Backend worker dropped reply channel")
                }
            }
        }
        "batch_actions" => {
            let actions = match arguments.get("actions").and_then(|v| v.as_array()) {
                Some(arr) => arr.clone(),
                _ => {
                    return JsonRpcResponse::error(
                        id,
                        -32602,
                        "Missing or invalid 'actions' array parameter",
                    );
                }
            };

            let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
            if let Err(err) = commands.send(Command::Mcp(McpCommand::BatchActions {
                actions,
                reply: reply_tx,
            })) {
                return JsonRpcResponse::error(
                    id,
                    -32603,
                    format!("Failed to dispatch command to backend: {err}"),
                );
            }

            match reply_rx.await {
                Ok(Ok(results)) => {
                    let content = vec![json!({
                        "type": "text",
                        "text": serde_json::to_string_pretty(&results).unwrap_or_default()
                    })];
                    JsonRpcResponse::success(id, json!({ "content": content }))
                }
                Ok(Err(err)) => JsonRpcResponse::error(id, -32000, err),
                Err(_) => {
                    JsonRpcResponse::error(id, -32603, "Backend worker dropped reply channel")
                }
            }
        }
        unknown => JsonRpcResponse::error(id, -32601, format!("Unknown tool: {unknown}")),
    }
}
