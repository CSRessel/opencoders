//! OpenCode SDK
//!
//! This module provides a type-safe, ergonomic Rust client for the OpenCode API.
//! It wraps the auto-generated OpenAPI client with additional functionality and
//! better error handling.

pub mod client;
pub mod error;
pub mod extensions;

// High-level exports for easy use
pub use client::OpenCodeClient;
pub use error::{OpenCodeError, Result};

// Re-export commonly used generated types for convenience
pub use opencode_sdk::models::{
    App, AssistantMessage, Config, Event, FilePart, Match, Message, Mode, Provider, Session, TextPart,
    ToolPart, UserMessage,
};

// Convenience type aliases
pub type SessionId = String;
pub type MessageId = String;
pub type ProviderId = String;
pub type ModelId = String;

// Re-export event stream functionality
pub use extensions::events::{EventStream, EventStreamHandle};
