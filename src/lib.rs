//! OpenCode Rust TUI
//! 
//! This crate provides a Terminal User Interface (TUI) for the OpenCode project.
//! It includes an SDK for communicating with the OpenCode server.

pub mod sdk;

// Re-export commonly used types for convenience
// Temporarily commented out due to generation issues
// pub use sdk::{OpenCodeClient, OpenCodeError, Result};