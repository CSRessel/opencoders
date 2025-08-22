mod app;
mod sdk;

fn main() -> app::Result<()> {
    // Install color-eyre for enhanced error reporting
    // This must be the very first operation to ensure proper error handling
    color_eyre::install().expect("Failed to install color-eyre");

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
