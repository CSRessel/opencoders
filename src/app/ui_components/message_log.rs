use opencode_sdk::models::{
    GetSessionByIdMessage200ResponseInner, Message, Part, TextPart, UserMessage,
};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Widget},
    Frame,
};

#[derive(Debug, Clone, PartialEq)]
pub struct MessageLog {
    message_log_scroll: u16,
    messages: Vec<GetSessionByIdMessage200ResponseInner>,
}

// pub fn render_message_log(frame: &mut Frame, rect: Rect, model: &Model) {
// }

impl MessageLog {
    pub fn new() -> Self {
        Self {
            message_log_scroll: 0,
            messages: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    pub fn message_log_scroll(&self) -> &u16 {
        &self.message_log_scroll
    }

    pub fn move_message_log_scroll(&mut self, direction: &i16) {
        let new_scroll = self.message_log_scroll as i16 + direction;
        self.message_log_scroll = new_scroll.max(0) as u16;
    }

    pub fn create_and_push_user_message(&mut self, submitted_text: &String) {
        let user_message = UserMessage {
            id: "".to_string(),
            session_id: "".to_string(),
            role: "user".to_string(),
            time: Default::default(),
        };

        let text_part = TextPart {
            id: "".to_string(),
            session_id: "".to_string(),
            message_id: "".to_string(),
            r#type: "text".to_string(),
            text: submitted_text.clone(),
            synthetic: None,
            time: None,
        };

        let message_container = GetSessionByIdMessage200ResponseInner {
            info: Box::new(Message::User(Box::new(user_message))),
            parts: vec![Part::Text(Box::new(text_part))],
        };

        self.messages.push(message_container);
    }

    fn display_message_list(&self) -> Vec<String> {
        self.messages
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
            .collect()
    }
}

impl Widget for &MessageLog {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let messages = self.display_message_list();

        let message_log = Paragraph::new(messages.join("\n"))
            .block(Block::default().borders(Borders::ALL).title("Message Log"))
            .scroll((self.message_log_scroll, 0));

        message_log.render(area, buf);
    }
}

impl Default for MessageLog {
    fn default() -> Self {
        Self::new()
    }
}
