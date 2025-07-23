use crate::app::{
    tea_model::{AppState, ConnectionStatus, Model},
    ui_components::create_welcome_text,
};
use core::error;
use ratatui::{
    crossterm,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::Paragraph,
    Frame,
};
use std::io::{self, Write};

fn calculate_content_width(terminal_width: u16) -> u16 {
    let min_width = 80;
    let ninety_percent = (terminal_width * 90) / 100;
    min_width.max(ninety_percent).min(terminal_width)
}

pub fn view_manual(model: &Model) -> Result<(), Box<dyn std::error::Error>> {
    // Handle any prints that precede the dynamic TUI interface

    crossterm::terminal::disable_raw_mode()?;
    // Move cursor up outside the TUI height
    crossterm::execute!(io::stdout(), crossterm::cursor::MoveUp(model.init.height()),)?;

    match model.state {
        AppState::TextEntry => render_manual_history(&model)?,
        AppState::Welcome => {}
        AppState::ConnectingToServer => {}
        AppState::InitializingSession => {}
        AppState::ConnectionError(_) => {}
        AppState::Quit => {}
    };

    // Move cursor back down to TUI
    crossterm::execute!(
        io::stdout(),
        crossterm::cursor::MoveDown(model.init.height())
    )?;
    crossterm::terminal::enable_raw_mode()?;
    Ok(())
}

fn render_manual_history(model: &Model) -> Result<(), Box<dyn std::error::Error>> {
    let messages = model.messages_needing_stdout_print();

    for message in messages {
        // TODO: handle multiple scrolls for multi-line wrapping
        crossterm::execute!(io::stdout(), crossterm::terminal::ScrollUp(1),)?;
        // Go to start of line
        crossterm::execute!(io::stdout(), crossterm::cursor::MoveToColumn(0))?;
        print!("> {}", message);
    }

    Ok(())
}

pub fn view(model: &Model, frame: &mut Frame) {
    match &model.state {
        AppState::Welcome => render_welcome_screen(model, frame),
        AppState::ConnectingToServer => render_connecting_screen(model, frame),
        AppState::InitializingSession => render_initializing_session_screen(model, frame),
        AppState::TextEntry => render_text_entry_screen(model, frame),
        AppState::ConnectionError(error) => render_error_screen(model, frame, error),
        AppState::Quit => {} // No rendering needed for quit state
    };
}

pub fn view_clear(_model: &Model, frame: &mut Frame) {
    // Write an empty frame to force full redraw of all cells
    frame.render_widget(Paragraph::new(""), frame.area());
}

fn render_welcome_screen(model: &Model, frame: &mut Frame) {
    let text = Text::from(
        "Press Enter to start text input, Tab to toggle inline, 'q' or 'Esc' to exit...",
    );
    let paragraph = Paragraph::new(text);

    if model.init.inline_mode() {
        let vertical_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(1)])
            .split(frame.area());
        frame.render_widget(paragraph, vertical_chunks[1]);
    } else {
        let constraints = vec![Constraint::Length(4), Constraint::Length(2)];

        let vertical_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(frame.area());

        frame.render_widget(create_welcome_text(), vertical_chunks[0]);
        frame.render_widget(paragraph, vertical_chunks[1]);
    };
}

fn render_text_entry_screen(model: &Model, frame: &mut Frame) {
    let terminal_width = frame.area().width;
    let content_width = calculate_content_width(terminal_width);
    let left_padding = (terminal_width.saturating_sub(content_width)) / 2;
    let right_padding = terminal_width.saturating_sub(content_width + left_padding);

    // Create horizontal layout for centering
    let horizontal_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(left_padding),
            Constraint::Length(content_width),
            Constraint::Length(right_padding),
        ])
        .split(frame.area());

    let mut content_area = horizontal_chunks[1];

    let input_height = 3;

    if model.init.inline_mode() {
        // Render only the text input for inline mode
        content_area.height = input_height;
        frame.render_widget(&model.text_input, content_area);
    } else {
        // Create vertical layout for message log and input box
        let vertical_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),               // Message log
                Constraint::Length(input_height), // Input box
            ])
            .split(content_area);

        frame.render_widget(&model.message_log, vertical_chunks[0]);
        frame.render_widget(&model.text_input, vertical_chunks[1]);
    }
}

fn render_connecting_screen(model: &Model, frame: &mut Frame) {
    let text = Text::from(vec![
        Line::from("Connecting to OpenCode server..."),
        Line::from(""),
        Line::from("Looking for running OpenCode processes..."),
        Line::from("Press 'q' or 'Esc' to cancel"),
    ]);
    let paragraph = Paragraph::new(text).style(Style::default().fg(Color::Yellow));

    if model.init.inline_mode() {
        let vertical_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(4)])
            .split(frame.area());
        frame.render_widget(paragraph, vertical_chunks[1]);
    } else {
        let vertical_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(4), Constraint::Min(0)])
            .split(frame.area());
        frame.render_widget(paragraph, vertical_chunks[1]);
    }
}

fn render_initializing_session_screen(model: &Model, frame: &mut Frame) {
    let client_url = model.client().map(|c| c.base_url()).unwrap_or("unknown");
    
    let text = Text::from(vec![
        Line::from("Initializing session..."),
        Line::from(""),
        Line::from(format!("Connected to: {}", client_url)),
        Line::from("Setting up your coding session..."),
        Line::from("Press 'q' or 'Esc' to cancel"),
    ]);
    let paragraph = Paragraph::new(text).style(Style::default().fg(Color::Blue));

    if model.init.inline_mode() {
        let vertical_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(5)])
            .split(frame.area());
        frame.render_widget(paragraph, vertical_chunks[1]);
    } else {
        let vertical_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(5), Constraint::Min(0)])
            .split(frame.area());
        frame.render_widget(paragraph, vertical_chunks[1]);
    }
}

fn render_error_screen(model: &Model, frame: &mut Frame, error: &str) {
    let text = Text::from(vec![
        Line::from("Connection Error"),
        Line::from(""),
        Line::from(error),
        Line::from(""),
        Line::from("Suggestions:"),
        Line::from("• Make sure OpenCode server is running"),
        Line::from("• Check OPENCODE_SERVER_URL environment variable"),
        Line::from("• Try running: opencode serve"),
        Line::from(""),
        Line::from("Press 'r' to retry, 'q' or 'Esc' to quit"),
    ]);
    let paragraph = Paragraph::new(text).style(Style::default().fg(Color::Red));

    if model.init.inline_mode() {
        let vertical_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(10)])
            .split(frame.area());
        frame.render_widget(paragraph, vertical_chunks[1]);
    } else {
        let vertical_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(10), Constraint::Min(0)])
            .split(frame.area());
        frame.render_widget(paragraph, vertical_chunks[1]);
    }
}
