//! Application-specific error types for the OpenCode TUI
//!
//! This module provides structured error handling for the TUI application,
//! enabling intelligent error recovery and clear error reporting.

use crate::sdk::OpenCodeError;
use color_eyre::{Section, SectionExt};
use eyre::{Report, WrapErr};

/// Result type alias for application operations
pub type Result<T> = eyre::Result<T>;

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

/// Extension trait for adding recovery strategy context to errors
pub trait RecoveryExt {
    /// Determine the appropriate recovery strategy for this error
    fn recovery_strategy(&self) -> RecoveryStrategy;
    
    /// Check if this error should cause the application to exit
    fn is_fatal(&self) -> bool {
        matches!(self.recovery_strategy(), RecoveryStrategy::Exit)
    }
    
    /// Check if this error can be retried
    fn is_retryable(&self) -> bool {
        matches!(self.recovery_strategy(), RecoveryStrategy::Retry)
    }
}

impl RecoveryExt for Report {
    fn recovery_strategy(&self) -> RecoveryStrategy {
        // Try to downcast to specific error types to determine recovery strategy
        if let Some(sdk_err) = self.downcast_ref::<OpenCodeError>() {
            if sdk_err.is_retryable() {
                return RecoveryStrategy::Retry;
            } else if sdk_err.is_client_error() {
                return RecoveryStrategy::Exit;
            } else {
                return RecoveryStrategy::Exit;
            }
        }
        
        if let Some(io_err) = self.downcast_ref::<std::io::Error>() {
            return match io_err.kind() {
                std::io::ErrorKind::TimedOut 
                | std::io::ErrorKind::Interrupted 
                | std::io::ErrorKind::WouldBlock => RecoveryStrategy::Retry,
                std::io::ErrorKind::BrokenPipe => RecoveryStrategy::Exit,
                _ => RecoveryStrategy::Exit,
            };
        }
        
        if let Some(_) = self.downcast_ref::<serde_json::Error>() {
            return RecoveryStrategy::Exit;
        }
        
        // Check error message for specific contexts
        let error_str = self.to_string().to_lowercase();
        if error_str.contains("terminal") {
            RecoveryStrategy::RestartTerminal
        } else if error_str.contains("logger") || error_str.contains("logging") {
            RecoveryStrategy::Ignore
        } else if error_str.contains("configuration") || error_str.contains("config") {
            RecoveryStrategy::Exit
        } else if error_str.contains("async task") || error_str.contains("event processing") {
            RecoveryStrategy::Ignore
        } else {
            // Default strategy
            RecoveryStrategy::Exit
        }
    }
}

/// Helper functions for creating contextual errors
pub mod context {
    use super::*;
    
    /// Create a terminal initialization error with recovery context
    pub fn terminal_init(message: impl Into<String>) -> Report {
        eyre::eyre!("{}", message.into())
            .with_section(|| "Terminal initialization failed".header("Error Type:"))
            .with_section(|| "Try restarting the terminal or checking terminal capabilities".header("Suggestion:"))
    }
    
    /// Create a configuration error with recovery context
    pub fn configuration(message: impl Into<String>) -> Report {
        eyre::eyre!("{}", message.into())
            .with_section(|| "Configuration error".header("Error Type:"))
            .with_section(|| "Check your configuration file and environment variables".header("Suggestion:"))
    }
    
    /// Create an async task error with recovery context
    pub fn async_task(message: impl Into<String>) -> Report {
        eyre::eyre!("{}", message.into())
            .with_section(|| "Async task failed".header("Error Type:"))
            .with_section(|| "This error will be logged but the application will continue".header("Recovery:"))
    }
    
    /// Create an event processing error with recovery context
    pub fn event_processing(message: impl Into<String>) -> Report {
        eyre::eyre!("{}", message.into())
            .with_section(|| "Event processing failed".header("Error Type:"))
            .with_section(|| "This error will be logged but the application will continue".header("Recovery:"))
    }
}

/// Extension trait for adding terminal-specific context to errors
pub trait TerminalErrorExt {
    /// Add terminal state context to an error
    fn with_terminal_context(self, raw_mode: bool, alternate_screen: bool) -> Report;
}

impl<E> TerminalErrorExt for E 
where
    E: Into<Report>,
{
    fn with_terminal_context(self, raw_mode: bool, alternate_screen: bool) -> Report {
        self.into()
            .with_section(move || format!("Raw mode: {}", raw_mode).header("Terminal State:"))
            .with_section(move || format!("Alternate screen: {}", alternate_screen).header("Screen Mode:"))
    }
}

/// Extension trait for adding SDK-specific context to errors
pub trait SdkErrorExt {
    /// Add SDK operation context to an error
    fn with_sdk_context(self, operation: &str, endpoint: Option<&str>) -> Report;
}

impl<E> SdkErrorExt for E 
where
    E: Into<Report>,
{
    fn with_sdk_context(self, operation: &str, endpoint: Option<&str>) -> Report {
        let mut report = self.into()
            .with_section(move || operation.to_string().header("SDK Operation:"));
        
        if let Some(endpoint) = endpoint {
            report = report.with_section(move || endpoint.to_string().header("Endpoint:"));
        }
        
        report
    }
}