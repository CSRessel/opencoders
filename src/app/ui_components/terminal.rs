use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal, TerminalOptions, Viewport};
use std::io::{self, Write};

pub struct TerminalGuard {
    inline_mode: bool,
}

impl TerminalGuard {
    pub fn new(
        inline_mode: bool,
    ) -> Result<(Self, Terminal<CrosstermBackend<io::Stdout>>), Box<dyn std::error::Error>> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        // execute!(stdout, EnableMouseCapture)?;

        if !inline_mode {
            execute!(stdout, EnterAlternateScreen)?;
        }

        let backend = CrosstermBackend::new(stdout);

        let viewport = if inline_mode {
            Viewport::Inline(5)
        } else {
            Viewport::Fullscreen
        };

        let mut terminal = Terminal::with_options(backend, TerminalOptions { viewport })?;

        // Clear the terminal and hide cursor
        terminal.clear()?;
        terminal.hide_cursor()?;

        let guard = TerminalGuard { inline_mode };

        Ok((guard, terminal))
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let mut stdout = io::stdout();
        // let _ = execute!(stdout, DisableMouseCapture);
        if !self.inline_mode {
            let _ = execute!(stdout, LeaveAlternateScreen);
        }
        let _ = stdout.flush();
    }
}
