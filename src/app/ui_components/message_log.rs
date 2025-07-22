use opencode_sdk::models::{
    GetSessionByIdMessage200ResponseInner, Message, Part, TextPart, UserMessage,
};
use ratatui::{
    buffer::Buffer,
    layout::{Margin, Rect},
    style::{Color, Style, Stylize},
    symbols::scrollbar,
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget, Widget},
    Frame,
};

#[derive(Debug, Clone, PartialEq)]
pub struct MessageLog {
    message_log_scroll: u16,
    messages: Vec<GetSessionByIdMessage200ResponseInner>,
    pub vertical_scroll_state: ScrollbarState,
    pub horizontal_scroll_state: ScrollbarState,
    vertical_scroll: usize,
    horizontal_scroll: usize,
}

// pub fn render_message_log(frame: &mut Frame, rect: Rect, model: &Model) {
// }

impl MessageLog {
    pub fn new() -> Self {
        Self {
            message_log_scroll: 0,
            messages: Vec::new(),
            vertical_scroll_state: ScrollbarState::default(),
            horizontal_scroll_state: ScrollbarState::default(),
            vertical_scroll: 0,
            horizontal_scroll: 0,
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
        
        // Update vertical scroll state
        self.vertical_scroll = self.message_log_scroll as usize;
        self.vertical_scroll_state = self.vertical_scroll_state.position(self.vertical_scroll);
    }
    
    pub fn scroll_horizontal(&mut self, direction: i16) {
        let new_scroll = self.horizontal_scroll as i16 + direction;
        self.horizontal_scroll = new_scroll.max(0) as usize;
        self.horizontal_scroll_state = self.horizontal_scroll_state.position(self.horizontal_scroll);
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
        let content = messages.join("\n");
        
        // Calculate content dimensions
        let content_lines = messages.len();
        let longest_line_length = messages.iter()
            .map(|line| line.len())
            .max()
            .unwrap_or(0);
        
        // Create a mutable clone for scrollbar state updates
        let mut message_log = self.clone();
        message_log.vertical_scroll_state = message_log.vertical_scroll_state.content_length(content_lines);
        message_log.horizontal_scroll_state = message_log.horizontal_scroll_state.content_length(longest_line_length);

        let paragraph = Paragraph::new(content)
            .block(Block::default().borders(Borders::ALL).title("Message Log".bold()).gray())
            .scroll((self.message_log_scroll, self.horizontal_scroll as u16));

        paragraph.render(area, buf);
        
        // Only render vertical scrollbar if content is taller than the available area
        if content_lines > (area.height.saturating_sub(2)) as usize {
            let vertical_scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .symbols(scrollbar::VERTICAL)
                .begin_symbol(None)
                .end_symbol(None);
            
            let scrollbar_area = area.inner(Margin {
                vertical: 1,
                horizontal: 0,
            });
            
            vertical_scrollbar.render(scrollbar_area, buf, &mut message_log.vertical_scroll_state);
        }
        
        // Only render horizontal scrollbar if content is wider than the available area
        if longest_line_length > (area.width.saturating_sub(2)) as usize {
            let horizontal_scrollbar = Scrollbar::new(ScrollbarOrientation::HorizontalBottom)
                .symbols(scrollbar::HORIZONTAL)
                .thumb_symbol("🬋")
                .begin_symbol(None)
                .end_symbol(None);
            
            let scrollbar_area = area.inner(Margin {
                vertical: 0,
                horizontal: 1,
            });
            
            horizontal_scrollbar.render(scrollbar_area, buf, &mut message_log.horizontal_scroll_state);
        }
    }
}

impl Default for MessageLog {
    fn default() -> Self {
        Self::new()
    }
}
