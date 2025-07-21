use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::Paragraph,
    Frame,
};
use crate::app::model::{Model, AppState};

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
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(frame.area());

    // Main content area
    if let Some(ref last_input) = model.last_input {
        let response_text = format!("You entered: {}", last_input);
        let paragraph = Paragraph::new(response_text);
        frame.render_widget(paragraph, chunks[0]);
    }

    // Text input at bottom
    frame.render_widget(&model.text_input, chunks[1]);
}

fn create_opencoders_ascii_art() -> Text<'static> {
    #[rustfmt::skip]
    let letters = vec![
        vec!["▄▀▀█", "█░░█", "▀▀▀ "], // o
        vec!["▄▀▀█", "█░░█", "█▀▀ "], // p
        vec!["▄▀▀▀", "█▀▀▀", "▀▀▀▀"], // e
        vec!["█▀▀▄", "█░░█", "▀  ▀"], // n
        vec!["▄▀▀▀", "█░░░", "▀▀▀▀"], // c
        vec!["▄▀▀█", "█░░█", "▀▀▀ "], // o
        vec!["█▀▀▄", "█░░█", "▀▀▀ "], // d
        vec!["▄▀▀▀", "█▀▀▀", "▀▀▀▀"], // e
        vec!["█▀▀█", "█▀▀▄", "▀  ▀"], // r
        vec!["▄▀▀▀", "▀▀▀█", "▀▀▀ "], // s
    ];

    let colors = vec![
        Color::Gray, Color::Gray, Color::Gray, Color::Gray, Color::White,
        Color::White, Color::White, Color::White, Color::Gray, Color::Gray,
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
    lines.push(Line::from("Press Enter to start text input, 'q' or 'Esc' to exit..."));
    Text::from(lines)
}