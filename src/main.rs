mod app;
mod sdk;

use std::panic;
use crossterm;

fn main() -> Result<(), Box<dyn std::error::Error>> {


    // Initialize logger
    app::logger::init().expect("Failed to initialize logger");
    tracing::info!("TUI application starting");

    // Log diagnostics in debug mode
    #[cfg(debug_assertions)]
    {
        tracing::debug!("Logger initialized");
    }

    let result = app::run();

    if let Err(ref e) = result {
        tracing::error!("Application error: {}", e);
    }

    tracing::info!("TUI application shutting down");
    result
}
