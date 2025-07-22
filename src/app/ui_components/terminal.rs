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
    ) -> Result<(Self, Terminal<CrosstermBackend<io::Stdout>>), Box<dyn std::error::Error>> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnableMouseCapture)?;

        if !init.inline_mode() {
            execute!(stdout, EnterAlternateScreen)?;
        }

        let backend = CrosstermBackend::new(stdout);

        let viewport = if init.inline_mode() {
            Viewport::Inline(init.height())
        } else {
            Viewport::Fullscreen
        };

        let mut terminal = Terminal::with_options(backend, TerminalOptions { viewport })?;

        // Clear the terminal and hide cursor
        terminal.clear()?;
        terminal.hide_cursor()?;

        let guard = TerminalGuard { init: init.clone() };

        Ok((guard, terminal))
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let mut stdout = io::stdout();
        let _ = execute!(stdout, DisableMouseCapture);
        if !self.init.inline_mode() {
            let _ = execute!(stdout, LeaveAlternateScreen);
        }
        let _ = stdout.flush();
    }
}
