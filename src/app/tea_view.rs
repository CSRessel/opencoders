use crate::app::{
    tea_model::{AppState, Model},
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
    crossterm::execute!(io::stdout(), crossterm::cursor::MoveUp(model.height),)?;

    match model.state {
        AppState::TextEntry => render_manual_history(&model)?,
        AppState::Welcome => {}
        AppState::Quit => {}
    };

    // Move cursor back down to TUI
    crossterm::execute!(io::stdout(), crossterm::cursor::MoveDown(model.height))?;
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
    match model.state {
        AppState::Welcome => render_welcome_screen(model, frame),
        AppState::TextEntry => render_text_entry_screen(model, frame),
        AppState::Quit => {} // No rendering needed for quit state
    };
}

pub fn view_clear(_model: &Model, frame: &mut Frame) {
    // Write an empty frame to force full redraw of all cells
    frame.render_widget(Paragraph::new(""), frame.area());
}

fn render_welcome_screen(model: &Model, frame: &mut Frame) {
    let text = Text::from("Press Enter to start text input, 'q' or 'Esc' to exit...");
    let paragraph = Paragraph::new(text);

    if model.inline_mode {
        frame.render_widget(paragraph, frame.area());
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
    if model.inline_mode {
        // Render only the text input for inline mode
        frame.render_widget(&model.text_input, frame.area());
    } else {
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

        let content_area = horizontal_chunks[1];

        // Create vertical layout for message log and input box
        let input_height = 3;
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
