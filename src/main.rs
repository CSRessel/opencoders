mod app;
mod sdk;

use std::panic;
use crossterm;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set custom panic hook for crash logging
    panic::set_hook(Box::new(|panic_info| {
        crash_log!("PANIC: {}", panic_info);
        
        // Attempt terminal cleanup
        let _ = crossterm::terminal::disable_raw_mode();
        let _ = crossterm::execute!(
            std::io::stderr(),
            crossterm::terminal::LeaveAlternateScreen
        );
        
        // Print crash log location to stderr (if terminal is restored)
        eprintln!("Application crashed. Check logs in /tmp/opencoders-crash-*");
    }));

    // Initialize logger
    app::logger::init_logger().expect("Failed to initialize logger");
    log_info!("TUI application starting");
    
    // Log diagnostics in debug mode
    #[cfg(debug_assertions)]
    {
        log_debug!("Logger diagnostics: {}", app::logger::get_logger_diagnostics());
    }
    
    let result = app::run();
    
    if let Err(ref e) = result {
        log_error!("Application error: {}", e);
    }
    
    log_info!("TUI application shutting down");
    result
}
