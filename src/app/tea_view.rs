use crate::app::{
    tea_model::{AppState, Model},
    ui_components::MessageHistory,
};
use ratatui::{
    layout::{Constraint, Direction, Layout, Alignment},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::Paragraph,
    Frame,
};

fn calculate_content_width(terminal_width: u16) -> u16 {
    let min_width = 80;
    let ninety_percent = (terminal_width * 90) / 100;
    min_width.max(ninety_percent).min(terminal_width)
}

pub fn view(model: &Model, frame: &mut Frame) {
    match model.state {
        AppState::Welcome => render_welcome_screen(frame),
        AppState::TextEntry => render_text_entry_screen(model, frame),
        AppState::Quit => {} // No rendering needed for quit state
    }
}

fn render_welcome_screen(frame: &mut Frame) {
    let text = create_opencoders_ascii_art();
    let paragraph = Paragraph::new(text);
    frame.render_widget(paragraph, frame.area());
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

    // Create vertical layout within the centered content area
    let input_height = 3;
    let history_height = content_area.height.saturating_sub(input_height + 1);
    
    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(history_height),
            Constraint::Length(1), // Spacing
            Constraint::Length(input_height),
        ])
        .split(content_area);

    // Render message history
    if !model.input_history.is_empty() {
        let message_history = MessageHistory::new(model.input_history.clone());
        frame.render_widget(&message_history, vertical_chunks[0]);
    }

    // Render text input
    frame.render_widget(&model.text_input, vertical_chunks[2]);
}

fn create_opencoders_ascii_art() -> Text<'static> {
    #[rustfmt::skip]
    let letters = vec![
        vec!["▄▀▀█",
             "█░░█",
             "▀▀▀ "], // o
        vec!["▄▀▀█",
             "█░░█",
             "█▀▀ "], // p
        vec!["▄▀▀▀",
             "█▀▀▀",
             "▀▀▀▀"], // e
        vec!["█▀▀▄",
             "█░░█",
             "▀  ▀"], // n
        vec!["▄▀▀▀",
             "█░░░",
             "▀▀▀▀"], // c
        vec!["▄▀▀█",
             "█░░█",
             "▀▀▀ "], // o
        vec!["█▀▀▄",
             "█░░█",
             "▀▀▀ "], // d
        vec!["▄▀▀▀",
             "█▀▀▀",
             "▀▀▀▀"], // e
        vec!["█▀▀█",
             "█▀▀▄",
             "▀  ▀"], // r
        vec!["▄▀▀▀",
             "▀▀▀█",
             "▀▀▀ "], // s
    ];

    let colors = vec![
        Color::Gray,
        Color::Gray,
        Color::Gray,
        Color::Gray,
        Color::White,
        Color::White,
        Color::White,
        Color::White,
        Color::Gray,
        Color::Gray,
    ];

    let mut lines = vec![Line::from("")];

    for row in 0..3 {
        let mut spans = Vec::new();

        for (letter_idx, letter) in letters.iter().enumerate() {
            let color = colors.get(letter_idx).unwrap_or(&Color::White);
            let style = Style::default().fg(*color);

            spans.push(Span::styled(letter[row], style));

            if letter_idx < letters.len() - 1 {
                spans.push(Span::raw(" "));
            }
        }

        lines.push(Line::from(spans));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(
        "Press Enter to start text input, 'q' or 'Esc' to exit...",
    ));
    Text::from(lines)
}

