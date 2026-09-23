//! Embedded Model Context Protocol (MCP) server for ZapFast.

pub mod server;
pub mod tools;
pub mod types;

pub use server::start_server;
pub use types::{McpChat, McpCommand, McpMessage, McpSendResult};
