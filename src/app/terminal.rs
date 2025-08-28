use crate::app::{
    error::Result,
    tea_model::{Model, ModelInit},
};
use crossterm::{
    event::{
        DisableMouseCapture, EnableMouseCapture, KeyboardEnhancementFlags,
        PushKeyboardEnhancementFlags,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use eyre::WrapErr;
use ratatui::{backend::CrosstermBackend, Terminal, TerminalOptions, Viewport};
use std::io::{self, stdout, Write};

pub fn align_crossterm_output_to_bottom(model: &Model) -> Result<()> {
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

/// Initialize the terminal with panic hook for automatic cleanup
pub fn init_terminal(
    init: &ModelInit,
    height: u16,
) -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    tracing::info!(
        "Initializing terminal - inline_mode: {}",
        init.inline_mode()
    );

    enable_raw_mode().wrap_err("Failed to enable raw mode")?;

    // Necessary for some terminals to report shift+enter and other modified keys
    let flags = KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
        | KeyboardEnhancementFlags::REPORT_EVENT_TYPES
        | KeyboardEnhancementFlags::REPORT_ALL_KEYS_AS_ESCAPE_CODES;
    crossterm::execute!(std::io::stdout(), PushKeyboardEnhancementFlags(flags))
        .wrap_err("Failed to push kb flags")?;

    let mut stdout = stdout();
    execute!(stdout, EnableMouseCapture).wrap_err("Failed to enable mouse capture")?;

    if !init.inline_mode() {
        tracing::debug!("Entering alternate screen mode");
        execute!(stdout, EnterAlternateScreen).wrap_err("Failed to enter alternate screen")?;
    } else {
        tracing::debug!("Using inline mode with height: {}", height);
    }

    // Set up panic hook for automatic terminal restoration
    set_panic_hook(init.clone(), height);

    let backend = CrosstermBackend::new(stdout);

    let viewport = if init.inline_mode() {
        Viewport::Inline(height)
    } else {
        Viewport::Fullscreen
    };

    let mut terminal = Terminal::with_options(backend, TerminalOptions { viewport })
        .wrap_err("Failed to create terminal")?;

    // Clear the terminal and hide cursor
    terminal.clear().wrap_err("Failed to clear terminal")?;
    terminal.hide_cursor().wrap_err("Failed to hide cursor")?;

    tracing::info!("Terminal initialized successfully");
    Ok(terminal)
}

/// Set panic hook to ensure terminal cleanup on panic
fn set_panic_hook(init: ModelInit, height: u16) {
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = restore_terminal(&init, height); // ignore any errors as we are already failing
        hook(panic_info);
    }));
}

/// Restore the terminal to its original state
pub fn restore_terminal(init: &ModelInit, height: u16) -> io::Result<()> {
    tracing::info!("Restoring terminal - inline_mode: {}", init.inline_mode());

    // Disable raw mode first
    if let Err(e) = disable_raw_mode() {
        tracing::error!("Failed to disable raw mode during restore: {}", e);
    }

    let mut stdout = stdout();

    // Disable mouse capture
    if let Err(e) = execute!(stdout, DisableMouseCapture) {
        tracing::error!("Failed to disable mouse capture during restore: {}", e);
    }

    // Handle screen mode restoration
    if !init.inline_mode() {
        tracing::debug!("Leaving alternate screen mode");
        execute!(stdout, LeaveAlternateScreen)?;
    } else {
        // For inline mode, ensure proper cursor positioning and screen clearing
        // to prevent overlap with error messages
        if let Ok((cols, rows)) = crossterm::terminal::size() {
            // Clear from cursor position down to prevent overlap
            execute!(
                stdout,
                crossterm::cursor::MoveTo(0, rows.saturating_sub(height)),
                crossterm::terminal::Clear(crossterm::terminal::ClearType::FromCursorDown),
                crossterm::cursor::Show
            )?;
        }
    }

    // Ensure all output is flushed
    stdout.flush()?;

    tracing::info!("Terminal restore completed");
    Ok(())
}
