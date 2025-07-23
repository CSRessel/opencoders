//! OpenCode Rust TUI
//!
//! This crate provides a Terminal User Interface (TUI) for the OpenCode project.
//! It includes an SDK for communicating with the OpenCode server.

pub mod app;
pub mod sdk;

// Re-export commonly used types for convenience
pub use sdk::{OpenCodeClient, OpenCodeError, Result};

