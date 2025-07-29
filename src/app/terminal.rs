use crate::{log_debug, log_info, tui_error};
use crate::app::tea_model::ModelInit;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal, TerminalOptions, Viewport};
use std::io::{self, Write};

pub struct TerminalGuard {
    init: ModelInit,
}

impl TerminalGuard {
    pub fn new(
        init: &ModelInit,
        height: u16,
    ) -> Result<(Self, Terminal<CrosstermBackend<io::Stdout>>), Box<dyn std::error::Error>> {
        log_info!("Initializing terminal - inline_mode: {}", init.inline_mode());
        
        if let Err(e) = enable_raw_mode() {
            tui_error!("Failed to enable raw mode: {}", e);
            return Err(e.into());
        }
        
        let mut stdout = io::stdout();
        if let Err(e) = execute!(stdout, EnableMouseCapture) {
            tui_error!("Failed to enable mouse capture: {}", e);
            return Err(e.into());
        }

        if !init.inline_mode() {
            log_debug!("Entering alternate screen mode");
            if let Err(e) = execute!(stdout, EnterAlternateScreen) {
                tui_error!("Failed to enter alternate screen: {}", e);
                return Err(e.into());
            }
        } else {
            log_debug!("Using inline mode with height: {}", height);
        }

        let backend = CrosstermBackend::new(stdout);

        let viewport = if init.inline_mode() {
            Viewport::Inline(height)
        } else {
            Viewport::Fullscreen
        };

        let mut terminal = Terminal::with_options(backend, TerminalOptions { viewport })?;

        // Clear the terminal and hide cursor
        terminal.clear()?;
        terminal.hide_cursor()?;

        let guard = TerminalGuard { init: init.clone() };
        
        log_info!("Terminal initialized successfully");
        Ok((guard, terminal))
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        log_info!("Cleaning up terminal - inline_mode: {}", self.init.inline_mode());
        
        if let Err(e) = disable_raw_mode() {
            tui_error!("Failed to disable raw mode during cleanup: {}", e);
        }
        
        let mut stdout = io::stdout();
        if let Err(e) = execute!(stdout, DisableMouseCapture) {
            tui_error!("Failed to disable mouse capture during cleanup: {}", e);
        }
        
        if !self.init.inline_mode() {
            log_debug!("Leaving alternate screen mode");
            if let Err(e) = execute!(stdout, LeaveAlternateScreen) {
                tui_error!("Failed to leave alternate screen during cleanup: {}", e);
            }
        }
        
        if let Err(e) = stdout.flush() {
            tui_error!("Failed to flush stdout during cleanup: {}", e);
        }
        
        log_info!("Terminal cleanup completed");
    }
}
