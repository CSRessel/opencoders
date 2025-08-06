use crate::app::tea_model::{AppState, Model};
use crate::app::ui_components::Paragraph;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span, Text},
    Frame,
};

pub fn welcome_text_height() -> u16 {
    4
}

pub fn create_welcome_text() -> Text<'static> {
    #[rustfmt::skip]
    let letters = vec![
        vec!["▄▀▀█",
             "█  █",
             "▀▀▀ "], // o
        vec!["▄▀▀█",
             "█  █",
             "█▀▀ "], // p
        vec!["▄▀▀▀",
             "█▀▀▀",
             "▀▀▀▀"], // e
        vec!["█▀▀▄",
             "█  █",
             "▀  ▀"], // n
        vec!["▄▀▀▀",
             "█   ",
             "▀▀▀▀"], // c
        vec!["▄▀▀█",
             "█  █",
             "▀▀▀ "], // o
        vec!["█▀▀▄",
             "█  █",
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
        Color::Red,
        Color::Red,
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
    Text::from(lines)
}
