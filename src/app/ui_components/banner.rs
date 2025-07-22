use crate::app::tea_model::{AppState, Model};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::Paragraph,
    Frame,
};

pub fn create_welcome_text() -> Text<'static> {
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
