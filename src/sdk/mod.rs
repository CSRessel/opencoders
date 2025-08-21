//! OpenCode SDK
//!
//! This module provides a type-safe, ergonomic Rust client for the OpenCode API.
//! It wraps the auto-generated OpenAPI client with additional functionality and
//! better error handling.

pub mod client;
pub mod discovery;
pub mod error;
pub mod extensions;
pub mod session_manager;
// pub mod streams;

// High-level exports for easy use
pub use client::OpenCodeClient;
pub use discovery::{discover_opencode_server, DiscoveryConfig};
pub use error::{OpenCodeError, Result};
pub use session_manager::SessionManager;

// Re-export commonly used generated types for convenience
pub use opencode_sdk::models::{
    App, AssistantMessage, Config, Event, FilePart, FindText200ResponseInner as Match, Message, Model as Mode, Provider, Session,
    TextPart, ToolPart, UserMessage,
};

// Convenience type aliases
pub type SessionId = String;
pub type MessageId = String;
pub type ProviderId = String;
pub type ModelId = String;

// Re-export event stream functionality
pub use extensions::events::{EventStream, EventStreamHandle};

// Log level enum for the write_log function
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}
