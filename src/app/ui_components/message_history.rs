use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

#[derive(Debug, Clone)]
pub struct MessageHistory {
    messages: Vec<String>,
}

impl MessageHistory {
    pub fn new(messages: Vec<String>) -> Self {
        Self { messages }
    }

    pub fn visible_messages(&self, available_height: u16) -> &[String] {
        let max_messages = (available_height / 3).max(1) as usize;
        let start_idx = self.messages.len().saturating_sub(max_messages);
        &self.messages[start_idx..]
    }
}

impl Widget for &MessageHistory {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let visible_messages = self.visible_messages(area.height);
        let message_height = 3;
        
        for (i, message) in visible_messages.iter().enumerate() {
            let y_offset = i as u16 * message_height;
            if y_offset + message_height > area.height {
                break;
            }

            let message_area = Rect {
                x: area.x,
                y: area.y + y_offset,
                width: area.width,
                height: message_height,
            };

            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray));

            let paragraph = Paragraph::new(Line::from(Span::styled(
                message.clone(),
                Style::default().fg(Color::White),
            )))
            .block(block);

            paragraph.render(message_area, buf);
        }
    }
}