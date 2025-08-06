use crate::app::tea_model::{Model, ModelInit};
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

pub fn align_crossterm_output_to_bottom(model: &Model) -> Result<(), Box<dyn std::error::Error>> {
    let (_window_cols, window_rows) = crossterm::terminal::size()?;
    let (_start_col, start_row) = crossterm::cursor::position()?;
    let expected_start_row = window_rows.saturating_sub(model.config.height.saturating_add(1));
    if start_row < expected_start_row {
        crossterm::execute!(
            io::stdout(),
            crossterm::cursor::MoveTo(0, expected_start_row)
        )?;
    }
    Ok(())
}

impl TerminalGuard {
    pub fn new(
        init: &ModelInit,
        height: u16,
    ) -> Result<(Self, Terminal<CrosstermBackend<io::Stdout>>), Box<dyn std::error::Error>> {
        tracing::info!(
            "Initializing terminal - inline_mode: {}",
            init.inline_mode()
        );

        if let Err(e) = enable_raw_mode() {
            tracing::error!("Failed to enable raw mode: {}", e);
            return Err(e.into());
        }

        let mut stdout = io::stdout();
        if let Err(e) = execute!(stdout, EnableMouseCapture) {
            tracing::error!("Failed to enable mouse capture: {}", e);
            return Err(e.into());
        }

        if !init.inline_mode() {
            tracing::debug!("Entering alternate screen mode");
            if let Err(e) = execute!(stdout, EnterAlternateScreen) {
                tracing::error!("Failed to enter alternate screen: {}", e);
                return Err(e.into());
            }
        } else {
            tracing::debug!("Using inline mode with height: {}", height);
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

        tracing::info!("Terminal initialized successfully");
        Ok((guard, terminal))
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        tracing::info!(
            "Cleaning up terminal - inline_mode: {}",
            self.init.inline_mode()
        );

        if let Err(e) = disable_raw_mode() {
            tracing::error!("Failed to disable raw mode during cleanup: {}", e);
        }

        let mut stdout = io::stdout();
        if let Err(e) = execute!(stdout, DisableMouseCapture) {
            tracing::error!("Failed to disable mouse capture during cleanup: {}", e);
        }

        if !self.init.inline_mode() {
            tracing::debug!("Leaving alternate screen mode");
            if let Err(e) = execute!(stdout, LeaveAlternateScreen) {
                tracing::error!("Failed to leave alternate screen during cleanup: {}", e);
            }
        }

        if let Err(e) = stdout.flush() {
            tracing::error!("Failed to flush stdout during cleanup: {}", e);
        }

        tracing::info!("Terminal cleanup completed");
    }
}
