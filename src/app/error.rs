//! Application-specific error types for the OpenCode TUI
//!
//! This module provides structured error handling for the TUI application,
//! enabling intelligent error recovery and clear error reporting.

use crate::sdk::OpenCodeError;
use thiserror::Error;

/// Result type alias for application operations
pub type Result<T> = std::result::Result<T, AppError>;

/// Recovery strategy for different error types
#[derive(Debug, Clone, PartialEq)]
pub enum RecoveryStrategy {
    /// Error can be retried (network issues, temporary failures)
    Retry,
    /// Terminal needs to be restarted (terminal corruption)
    RestartTerminal,
    /// Application should exit gracefully
    Exit,
    /// Error should be logged but ignored
    Ignore,
}

/// Main error type for the TUI application
#[derive(Error, Debug)]
pub enum AppError {
    /// SDK operation failed
    #[error("SDK operation failed: {0}")]
    Sdk(#[from] OpenCodeError),

    /// Terminal operation failed
    #[error("Terminal operation failed: {0}")]
    Terminal(std::io::Error),

    /// IO operation failed
    #[error("IO operation failed: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialization/deserialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Logger initialization failed
    #[error("Logger initialization failed: {0}")]
    LoggerInit(#[from] anyhow::Error),

    /// Terminal initialization failed
    #[error("Terminal initialization failed: {0}")]
    TerminalInit(String),

    /// Application configuration error
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Async task failed
    #[error("Async task failed: {0}")]
    AsyncTask(String),

    /// Event processing error
    #[error("Event processing failed: {0}")]
    EventProcessing(String),
}

impl AppError {
    /// Determine the appropriate recovery strategy for this error
    pub fn recovery_strategy(&self) -> RecoveryStrategy {
        match self {
            Self::Sdk(e) if e.is_retryable() => RecoveryStrategy::Retry,
            Self::Sdk(e) if e.is_client_error() => RecoveryStrategy::Exit,
            Self::Sdk(_) => RecoveryStrategy::Exit,
            Self::Terminal(_) => RecoveryStrategy::RestartTerminal,
            Self::Io(e) => {
                match e.kind() {
                    std::io::ErrorKind::TimedOut 
                    | std::io::ErrorKind::Interrupted 
                    | std::io::ErrorKind::WouldBlock => RecoveryStrategy::Retry,
                    std::io::ErrorKind::BrokenPipe => RecoveryStrategy::Exit,
                    _ => RecoveryStrategy::Exit,
                }
            }
            Self::LoggerInit(_) => RecoveryStrategy::Ignore,
            Self::TerminalInit(_) => RecoveryStrategy::Exit,
            Self::Configuration(_) => RecoveryStrategy::Exit,
            Self::AsyncTask(_) => RecoveryStrategy::Ignore,
            Self::EventProcessing(_) => RecoveryStrategy::Ignore,
            Self::Serialization(_) => RecoveryStrategy::Exit,
        }
    }

    /// Check if this error should cause the application to exit
    pub fn is_fatal(&self) -> bool {
        matches!(self.recovery_strategy(), RecoveryStrategy::Exit)
    }

    /// Check if this error can be retried
    pub fn is_retryable(&self) -> bool {
        matches!(self.recovery_strategy(), RecoveryStrategy::Retry)
    }

    /// Create a terminal initialization error
    pub fn terminal_init(message: impl Into<String>) -> Self {
        Self::TerminalInit(message.into())
    }

    /// Create a configuration error
    pub fn configuration(message: impl Into<String>) -> Self {
        Self::Configuration(message.into())
    }

    /// Create an async task error
    pub fn async_task(message: impl Into<String>) -> Self {
        Self::AsyncTask(message.into())
    }

    /// Create an event processing error
    pub fn event_processing(message: impl Into<String>) -> Self {
        Self::EventProcessing(message.into())
    }
}

/// Extension trait for adding context to errors
pub trait ErrorContext<T> {
    /// Add context to an error
    fn with_context<F>(self, f: F) -> Result<T>
    where
        F: FnOnce() -> String;
}

impl<T, E> ErrorContext<T> for std::result::Result<T, E>
where
    E: Into<AppError>,
{
    fn with_context<F>(self, f: F) -> Result<T>
    where
        F: FnOnce() -> String,
    {
        self.map_err(|e| {
            let base_error = e.into();
            let context = f();
            match &base_error {
                AppError::Sdk(sdk_err) => AppError::AsyncTask(format!("{}: {}", context, sdk_err)),
                AppError::Terminal(term_err) => AppError::TerminalInit(format!("{}: {}", context, term_err)),
                AppError::Io(io_err) => AppError::AsyncTask(format!("{}: {}", context, io_err)),
                _ => base_error,
            }
        })
    }
}