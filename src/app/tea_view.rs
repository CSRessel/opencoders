use crate::app::tea_model::{AppState, Model};
use core::error;
use ratatui::{
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

pub fn view_prefix_inline(
    model: &Model,
    frame: &mut Frame,
) -> Result<(), Box<dyn std::error::Error>> {
    // Handle any printing before the dynamic TUI interface

    match model.state {
        AppState::TextEntry => render_text_entry_inline_prefix(model, frame)?,
        AppState::Welcome => {}
        AppState::Quit => {}
    };
    Ok(())
}

pub fn view(model: &Model, frame: &mut Frame) {
    match model.state {
        AppState::Welcome => render_welcome_screen(frame),
        AppState::TextEntry => render_text_entry_screen(model, frame),
        AppState::Quit => {} // No rendering needed for quit state
    };
}

fn render_welcome_screen(frame: &mut Frame) {
    let text = Text::from("Press Enter to start text input, 'q' or 'Esc' to exit...");
    let paragraph = Paragraph::new(text);
    frame.render_widget(paragraph, frame.area());
}

fn force_manual_carriage_return_inline(
    lines: Option<u16>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Restart line first
    print!("\r");
    // Scroll down once for space
    crossterm::execute!(
        io::stdout(),
        crossterm::terminal::ScrollUp(lines.unwrap_or(1)),
        crossterm::cursor::MoveUp(lines.unwrap_or(1)),
    )?;
    Ok(())
}

fn print_inline_overflow_messages(messages: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    // Simple approach: temporarily exit raw mode, print to stdout, then re-enter
    crossterm::terminal::disable_raw_mode()?;
    // Move cursor up height many lines
    force_manual_carriage_return_inline(Some(3))?;

    for message in messages {
        print!("> {}\n", message);
    }
    print!("\n");
    crossterm::terminal::enable_raw_mode()?;
    Ok(())
}

fn render_text_entry_inline_prefix(
    model: &Model,
    frame: &mut Frame,
) -> Result<(), Box<dyn std::error::Error>> {
    let messages_to_print = model.messages_needing_stdout_print();

    if messages_to_print.is_empty() {
        return Ok(());
    }

    // And rewrite an empty frame to force full redraw of all cells
    frame.render_widget(Paragraph::new(""), frame.area());

    print_inline_overflow_messages(messages_to_print)?;

    Ok(())
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

    let content_area = horizontal_chunks[1];

    // Create vertical layout for centering the input box
    let input_height = 3;

    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),               // Top spacing
            Constraint::Length(input_height), // Input box
            Constraint::Min(0),               // Bottom spacing
        ])
        .split(content_area);

    // Render only the text input - no history
    frame.render_widget(&model.text_input, vertical_chunks[1]);
}
