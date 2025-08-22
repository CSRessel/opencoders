mod app;
mod sdk;

use crossterm;
use std::panic;

fn main() -> app::Result<()> {
    // Initialize logger - keep guard alive for the duration of the program
    let _logger_guard = app::logger::init().expect("Failed to initialize logger");
    // Log diagnostics in debug mode
    #[cfg(debug_assertions)]
    {
        tracing::debug!("Logger initialized");
    }

    tracing::info!("TUI application starting");

    let result = app::run();

    if let Err(ref e) = result {
        tracing::error!("Application error: {}", e);
    }

    tracing::info!("TUI application shutting down");
    result
}
