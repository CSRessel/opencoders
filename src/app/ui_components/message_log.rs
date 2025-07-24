use crate::app::ui_components::MessagePart;
use opencode_sdk::models::{
    GetSessionByIdMessage200ResponseInner, Message, Part, TextPart, UserMessage,
};
use ratatui::{
    buffer::Buffer,
    layout::{Margin, Rect},
    style::{Color, Style, Stylize},
    symbols::scrollbar,
    text::{Line, Span, Text},
    widgets::{
        Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget,
        Widget,
    },
    Frame,
};

#[derive(Debug, Clone, PartialEq)]
pub struct MessageLog {
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

    pub fn move_message_log_scroll(&mut self, direction: &i16) {
        let content_lines = self.get_total_line_count();
        // Conservative estimate: assume minimum viewport of 10 lines
        let min_viewport_height = 10;

        let max_scroll = if content_lines > min_viewport_height {
            content_lines - min_viewport_height
        } else {
            0
        };

        let new_scroll = (self.vertical_scroll as i16 + direction)
            .max(0)
            .min(max_scroll as i16);
        self.vertical_scroll = new_scroll as usize;

        // Update vertical scroll state
        self.vertical_scroll_state = self.vertical_scroll_state.position(self.vertical_scroll);
    }

    pub fn scroll_horizontal(&mut self, direction: i16) {
        // Conservative estimate: assume minimum viewport of 50 characters
        let min_viewport_width = 50;
        let longest_line_length = self.calculate_longest_line_length();

        let max_scroll = if longest_line_length > min_viewport_width {
            longest_line_length - min_viewport_width
        } else {
            0
        };

        let new_scroll = (self.horizontal_scroll as i16 + direction)
            .max(0)
            .min(max_scroll as i16);
        self.horizontal_scroll = new_scroll as usize;
        self.horizontal_scroll_state = self
            .horizontal_scroll_state
            .position(self.horizontal_scroll);
    }

    pub fn scroll_vertical_with_viewport(&mut self, direction: i16, viewport_height: u16) {
        let content_lines = self.get_total_line_count();
        let available_height = viewport_height.saturating_sub(2) as usize; // Account for borders

        let max_scroll = if content_lines > available_height {
            content_lines - available_height
        } else {
            0
        };

        let new_scroll = (self.vertical_scroll as i16 + direction)
            .max(0)
            .min(max_scroll as i16);
        self.vertical_scroll = new_scroll as usize;

        // Update vertical scroll state
        self.vertical_scroll_state = self.vertical_scroll_state.position(self.vertical_scroll);
    }

    pub fn scroll_horizontal_with_viewport(&mut self, direction: i16, viewport_width: u16) {
        let available_width = viewport_width.saturating_sub(2) as usize; // Account for borders
        let longest_line_length = self.calculate_longest_line_length();

        let max_scroll = if longest_line_length > available_width {
            longest_line_length - available_width
        } else {
            0
        };

        let new_scroll = (self.horizontal_scroll as i16 + direction)
            .max(0)
            .min(max_scroll as i16);
        self.horizontal_scroll = new_scroll as usize;
        self.horizontal_scroll_state = self
            .horizontal_scroll_state
            .position(self.horizontal_scroll);
    }

    pub fn scroll_to_bottom(&mut self) {
        let content_lines = self.get_total_line_count();
        // Set scroll to a large value - it will be constrained during render
        // This ensures we always attempt to scroll to the maximum possible position
        self.vertical_scroll = content_lines.saturating_sub(1).max(0);
        self.vertical_scroll_state = self.vertical_scroll_state.position(self.vertical_scroll);
    }

    pub fn scroll_to_bottom_with_viewport(&mut self, viewport_height: u16) {
        let content_lines = self.get_total_line_count();
        let available_height = viewport_height.saturating_sub(2) as usize; // Account for borders

        let max_scroll = if content_lines > available_height {
            content_lines - available_height
        } else {
            0
        };

        self.vertical_scroll = max_scroll;
        self.vertical_scroll_state = self.vertical_scroll_state.position(self.vertical_scroll);
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

        // Auto-scroll to bottom when new message is added
        self.scroll_to_bottom();
    }

    pub fn set_messages(&mut self, messages: Vec<GetSessionByIdMessage200ResponseInner>) {
        self.messages = messages;

        // Auto-scroll to bottom when new message is added
        self.scroll_to_bottom();
    }

    pub fn add_message(&mut self, message: GetSessionByIdMessage200ResponseInner) {
        self.messages.push(message);

        // Auto-scroll to bottom when new message is added
        self.scroll_to_bottom();
    }

    fn render_message_content(&self) -> Text<'static> {
        let mut lines = Vec::new();

        for msg_container in &self.messages {
            let role = match *msg_container.info {
                Message::User(_) => "You",
                Message::Assistant(_) => "Assistant",
            };

            // Add role header
            lines.push(Line::from(vec![Span::styled(
                format!("{}: ", role),
                Style::default()
                    .fg(if role == "You" {
                        Color::Cyan
                    } else {
                        Color::Green
                    })
                    .bold(),
            )]));

            // Render each part using MessagePart component
            for part in &msg_container.parts {
                let message_part = MessagePart::new(part);
                let part_text = message_part.to_text();

                // Add each line from the part with proper indentation
                for line in part_text.lines {
                    let mut indented_spans = vec![Span::raw("  ")]; // 2-space indent
                    indented_spans.extend(line.spans);
                    lines.push(Line::from(indented_spans));
                }
            }

            // Add empty line between messages
            lines.push(Line::from(""));
        }

        Text::from(lines)
    }

    fn calculate_content_dimensions(&self) -> (usize, usize) {
        let content = self.render_message_content();
        let line_count = content.lines.len();
        let longest_line_length = content
            .lines
            .iter()
            .map(|line| {
                line.spans
                    .iter()
                    .map(|span| span.content.len())
                    .sum::<usize>()
            })
            .max()
            .unwrap_or(0);
        
        (line_count, longest_line_length)
    }

    fn get_total_line_count(&self) -> usize {
        let (line_count, _) = self.calculate_content_dimensions();
        line_count
    }

    fn calculate_longest_line_length(&self) -> usize {
        let (_, longest_line_length) = self.calculate_content_dimensions();
        longest_line_length
    }
}

impl Widget for &MessageLog {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let content = self.render_message_content();

        // Calculate content dimensions
        let (content_lines, longest_line_length) = self.calculate_content_dimensions();

        // Create a mutable clone for scrollbar state updates
        let mut message_log = self.clone();

        // Calculate scrollbar areas to match content length properly
        // let vertical_scrollbar_area = Rect {
        //     x: area.x + area.width - 1,
        //     y: area.y + 1,
        //     width: 1,
        //     height: if has_horizontal_scrollbar { area.height - 1 } else { area.height - 1 },
        // };
        let vertical_scrollbar_area = area.inner(Margin {
            vertical: 1,
            horizontal: 0,
        });
        let horizontal_scrollbar_area = area.inner(Margin {
            vertical: 0,
            horizontal: 1,
        });

        // Constrain scroll position based on actual viewport dimensions
        let available_height = area.height.saturating_sub(2) as usize; // Account for borders
        let available_width = area.width.saturating_sub(2) as usize; // Account for borders

        let max_vertical_scroll = if content_lines > available_height {
            content_lines - available_height
        } else {
            0
        };

        let max_horizontal_scroll = if longest_line_length > available_width {
            longest_line_length - available_width
        } else {
            0
        };

        // Constrain current scroll positions to viewport limits
        message_log.vertical_scroll = message_log.vertical_scroll.min(max_vertical_scroll);
        message_log.horizontal_scroll = message_log.horizontal_scroll.min(max_horizontal_scroll);

        // Set scrollbar states with proper content lengths and positions
        message_log.vertical_scroll_state = message_log
            .vertical_scroll_state
            .content_length(content_lines)
            .position(message_log.vertical_scroll);

        message_log.horizontal_scroll_state = message_log
            .horizontal_scroll_state
            .content_length(longest_line_length)
            .position(message_log.horizontal_scroll);

        let paragraph = Paragraph::new(content)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Message Log".bold())
                    .gray(),
            )
            .scroll((
                message_log.vertical_scroll as u16,
                message_log.horizontal_scroll as u16,
            ));

        paragraph.render(area, buf);

        // Only render vertical scrollbar if content is taller than the available area
        if content_lines > (area.height.saturating_sub(2)) as usize {
            let vertical_scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .symbols(scrollbar::VERTICAL)
                .begin_symbol(None)
                .end_symbol(None);

            vertical_scrollbar.render(
                vertical_scrollbar_area,
                buf,
                &mut message_log.vertical_scroll_state,
            );
        }

        // Only render horizontal scrollbar if content is wider than the available area
        if longest_line_length > (area.width.saturating_sub(2)) as usize {
            let horizontal_scrollbar = Scrollbar::new(ScrollbarOrientation::HorizontalBottom)
                .symbols(scrollbar::HORIZONTAL)
                .thumb_symbol("🬋")
                .begin_symbol(None)
                .end_symbol(None);

            horizontal_scrollbar.render(
                horizontal_scrollbar_area,
                buf,
                &mut message_log.horizontal_scroll_state,
            );
        }
    }
}

impl Default for MessageLog {
    fn default() -> Self {
        Self::new()
    }
}
