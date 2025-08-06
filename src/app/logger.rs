//! Optimized tracing configuration for the OpenCode TUI application.
//!
//! This module provides performance-optimized logging that:
//! 1. Prioritizes TUI performance by avoiding stdout/stderr output
//! 2. Provides granular, per-thread logging in debug builds
//! 3. Minimizes overhead in release builds
//! 4. Logs to files outside of the terminal interface
//!
//! ## Build Configuration
//!
//! ### Debug Builds (`cargo build`)
//! - Detailed logging with thread IDs, names, file locations, and line numbers
//! - Default level: `debug` for opencoders, `debug` for opencode-sdk
//! - Log file: `~/.opencode/logs/opencode-debug.log` (daily rotation)
//!
//! ### Release Builds (`cargo build --release`)
//! - Compact logging optimized for performance
//! - Default level: `info` for opencoders, `warn` for opencode-sdk
//! - Log file: `~/.opencode/logs/opencode.log` (daily rotation)
//!
//! ## Environment Variables
//!
//! - `OPENCODE_LOG_DIR`: Override log directory (default: `~/.opencode/logs`)
//! - `RUST_LOG`: Override log levels (e.g., `RUST_LOG=opencoders=trace`)

use anyhow::Result;
use std::path::PathBuf;
use tracing_appender::rolling;
use tracing_subscriber::{self, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};

pub fn init() -> Result<()> {
    let log_dir = get_log_directory();
    
    #[cfg(debug_assertions)]
    {
        init_debug_tracing(&log_dir)
    }
    #[cfg(not(debug_assertions))]
    {
        init_release_tracing(&log_dir)
    }
}

fn get_log_directory() -> PathBuf {
    if let Ok(dir) = std::env::var("OPENCODE_LOG_DIR") {
        PathBuf::from(dir)
    } else if let Some(home) = dirs::home_dir() {
        home.join(".opencode").join("logs")
    } else {
        PathBuf::from("/tmp/opencode")
    }
}

#[cfg(debug_assertions)]
fn init_debug_tracing(log_dir: &PathBuf) -> Result<()> {
    std::fs::create_dir_all(log_dir)?;
    
    let log_file = rolling::daily(log_dir, "opencode-debug.log");
    let (non_blocking_log_file, _guard) = tracing_appender::non_blocking(log_file);
    
    let file_layer = tracing_subscriber::fmt::layer()
        .with_writer(non_blocking_log_file)
        .with_ansi(false)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_file(true)
        .with_line_number(true)
        .with_target(true)
        .with_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("opencoders=debug,opencode_sdk=debug"))
        );

    tracing_subscriber::registry()
        .with(file_layer)
        .init();

    std::mem::forget(_guard);
    
    tracing::info!("Debug tracing initialized with detailed logging to: {}", log_dir.display());
    Ok(())
}

#[cfg(not(debug_assertions))]
fn init_release_tracing(log_dir: &PathBuf) -> Result<()> {
    std::fs::create_dir_all(log_dir)?;
    
    let log_file = rolling::daily(log_dir, "opencode.log");
    let (non_blocking_log_file, _guard) = tracing_appender::non_blocking(log_file);
    
    let file_layer = tracing_subscriber::fmt::layer()
        .with_writer(non_blocking_log_file)
        .with_ansi(false)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_file(false)
        .with_line_number(false)
        .with_target(false)
        .compact()
        .with_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("opencoders=info,opencode_sdk=warn"))
        );

    tracing_subscriber::registry()
        .with(file_layer)
        .init();

    std::mem::forget(_guard);
    
    tracing::info!("Release tracing initialized with optimized logging to: {}", log_dir.display());
    Ok(())
}
