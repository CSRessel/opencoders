use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::tea_model::Model;

pub fn render_message_log(frame: &mut Frame, rect: Rect, model: &Model) {
    let messages: Vec<String> = model
        .input_history
        .iter()
        .map(|msg| format!("You: {}", msg))
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
