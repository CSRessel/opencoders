use crate::app::tea_model::Model;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal, TerminalOptions, Viewport};
use std::io::{self, Write};

pub struct TerminalGuard {
    model: Model,
}

impl TerminalGuard {
    pub fn new(
        model: &Model,
    ) -> Result<(Self, Terminal<CrosstermBackend<io::Stdout>>), Box<dyn std::error::Error>> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnableMouseCapture)?;

        if !model.inline_mode {
            execute!(stdout, EnterAlternateScreen)?;
        }

        let backend = CrosstermBackend::new(stdout);

        let viewport = if model.inline_mode {
            Viewport::Inline(model.height)
        } else {
            Viewport::Fullscreen
        };

        let mut terminal = Terminal::with_options(backend, TerminalOptions { viewport })?;

        // Clear the terminal and hide cursor
        terminal.clear()?;
        terminal.hide_cursor()?;

        let guard = TerminalGuard {
            model: model.clone(),
        };

        Ok((guard, terminal))
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let mut stdout = io::stdout();
        let _ = execute!(stdout, DisableMouseCapture);
        if !self.model.inline_mode {
            let _ = execute!(stdout, LeaveAlternateScreen);
        }
        let _ = stdout.flush();
    }
}
