use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::tea_model::Model;
use opencode_sdk::models::{Message, Part};

pub fn render_message_log(frame: &mut Frame, rect: Rect, model: &Model) {
    let messages: Vec<String> = model
        .messages
        .iter()
        .map(|msg_container| {
            let role = match *msg_container.info {
                Message::User(_) => "You",
                Message::Assistant(_) => "Assistant",
            };

            let text_parts: Vec<String> = msg_container
                .parts
                .iter()
                .filter_map(|part| {
                    if let Part::Text(text_part) = part {
                        Some(text_part.text.clone())
                    } else {
                        None
                    }
                })
                .collect();

            format!("{}: {}", role, text_parts.join(" "))
        })
        .collect();

    let message_log = Paragraph::new(messages.join("\n"))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Message Log"),
        )
        .scroll((model.message_log_scroll, 0));

    frame.render_widget(message_log, rect);
}
