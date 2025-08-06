use crate::app::{
    ui_components::{Block, MessagePart, Paragraph},
    view_model_context::ViewModelContext,
};
use opencode_sdk::models::{
    GetSessionByIdMessage200ResponseInner, Message, Part, TextPart, UserMessage,
};
use ratatui::{
    buffer::Buffer,
    layout::{Margin, Rect},
    style::{Color, Style, Stylize},
    symbols::scrollbar,
    text::{Line, Span, Text},
    widgets::{Borders, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget, Widget},
    Frame,
};

#[derive(Debug, Clone, PartialEq)]
pub struct MessageLog {
    messages: Vec<GetSessionByIdMessage200ResponseInner>,
    pub vertical_scroll_state: ScrollbarState,
    pub horizontal_scroll_state: ScrollbarState,
    vertical_scroll: usize,
    horizontal_scroll: usize,
    // Content caching to avoid recalculation
    cached_content_lines: Option<usize>,
    cached_longest_line: Option<usize>,
    content_dirty: bool,
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
            cached_content_lines: None,
            cached_longest_line: None,
            content_dirty: true,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    pub fn scroll_vertical(&mut self, direction: &i16) {
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

        // Update vertical scroll state with content length
        self.vertical_scroll_state = self
            .vertical_scroll_state
            .content_length(content_lines)
            .position(self.vertical_scroll);
    }

    pub fn validate_scroll_position(&mut self, viewport_height: u16, viewport_width: u16) {
        let content_lines = self.get_total_line_count();
        let longest_line_length = self.calculate_longest_line_length();

        let available_height = viewport_height.saturating_sub(2) as usize; // Account for borders
        let available_width = viewport_width.saturating_sub(2) as usize; // Account for borders

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
        self.vertical_scroll = self.vertical_scroll.min(max_vertical_scroll);
        self.horizontal_scroll = self.horizontal_scroll.min(max_horizontal_scroll);

        // Update scrollbar states with proper content lengths and positions
        self.vertical_scroll_state = self
            .vertical_scroll_state
            .content_length(content_lines)
            .position(self.vertical_scroll);

        self.horizontal_scroll_state = self
            .horizontal_scroll_state
            .content_length(longest_line_length)
            .position(self.horizontal_scroll);
    }

    pub fn refresh_scrollbar_states(&mut self) {
        let content_lines = self.get_total_line_count();
        let longest_line_length = self.calculate_longest_line_length();

        // Update scrollbar states with current content dimensions
        self.vertical_scroll_state = self
            .vertical_scroll_state
            .content_length(content_lines)
            .position(self.vertical_scroll);

        self.horizontal_scroll_state = self
            .horizontal_scroll_state
            .content_length(longest_line_length)
            .position(self.horizontal_scroll);
    }

    pub fn scroll_horizontal(&mut self, direction: i16) {
        // Conservative estimate: assume minimum viewport of 50 characters
        let min_viewport_width = 50; // Account for borders
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

        // Update horizontal scroll state with content length
        self.horizontal_scroll_state = self
            .horizontal_scroll_state
            .content_length(longest_line_length)
            .position(self.horizontal_scroll);
    }

    pub fn touch_scroll(&mut self) {
        // Sync to bottom, then update the scroll state
        let content_lines = self.get_total_line_count();
        self.vertical_scroll = content_lines.saturating_sub(1).max(0);
        self.horizontal_scroll = 0;

        // Refresh scrollbar states after changing position
        self.refresh_scrollbar_states();
    }

    pub fn set_messages(&mut self, messages: Vec<GetSessionByIdMessage200ResponseInner>) {
        self.messages = messages;
        self.mark_content_dirty();

        // Auto-scroll to bottom when new message is added
        self.touch_scroll();
    }

    pub fn add_message(&mut self, message: GetSessionByIdMessage200ResponseInner) {
        self.messages.push(message);
        self.mark_content_dirty();

        // Auto-scroll to bottom when new message is added
        self.touch_scroll();
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

    fn mark_content_dirty(&mut self) {
        self.content_dirty = true;
        self.cached_content_lines = None;
        self.cached_longest_line = None;
    }

    fn calculate_content_dimensions(&mut self) -> (usize, usize) {
        if !self.content_dirty
            && self.cached_content_lines.is_some()
            && self.cached_longest_line.is_some()
        {
            return (
                self.cached_content_lines.unwrap(),
                self.cached_longest_line.unwrap(),
            );
        }

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

        // Cache the results
        self.cached_content_lines = Some(line_count);
        self.cached_longest_line = Some(longest_line_length);
        self.content_dirty = false;

        (line_count, longest_line_length)
    }

    fn get_total_line_count(&mut self) -> usize {
        let (line_count, _) = self.calculate_content_dimensions();
        line_count
    }

    fn calculate_longest_line_length(&mut self) -> usize {
        let (_, longest_line_length) = self.calculate_content_dimensions();
        longest_line_length
    }
}

impl Widget for &MessageLog {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let content = self.render_message_content();

        // Always calculate dimensions from the actual content being rendered
        // This ensures content and scroll state are perfectly synchronized
        let content_lines = content.lines.len();
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

        let vertical_scrollbar_area = area.inner(Margin {
            vertical: 1,
            horizontal: 0,
        });
        let horizontal_scrollbar_area = area.inner(Margin {
            vertical: 0,
            horizontal: 1,
        });

        // Use current scroll positions directly from the model (no mutation)
        let constrained_vertical_scroll = {
            let available_height = area.height.saturating_sub(2) as usize;
            let max_vertical_scroll = if content_lines > available_height {
                content_lines - available_height
            } else {
                0
            };
            self.vertical_scroll.min(max_vertical_scroll)
        };

        let constrained_horizontal_scroll = {
            let available_width = area.width.saturating_sub(2) as usize;
            let max_horizontal_scroll = if longest_line_length > available_width {
                longest_line_length - available_width
            } else {
                0
            };
            self.horizontal_scroll.min(max_horizontal_scroll)
        };

        // Create scrollbar states for rendering using fresh content dimensions
        // This ensures scrollbar state matches the actual content being rendered
        let mut vertical_scrollbar_state = self
            .vertical_scroll_state
            .content_length(content_lines)
            .position(constrained_vertical_scroll);

        let mut horizontal_scrollbar_state = self
            .horizontal_scroll_state
            .content_length(longest_line_length)
            .position(constrained_horizontal_scroll);

        let paragraph = Paragraph::new(content)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Message Log".bold())
                    .gray(),
            )
            .scroll((
                constrained_vertical_scroll as u16,
                constrained_horizontal_scroll as u16,
            ));

        paragraph.render(area, buf);

        // Only render vertical scrollbar if content is taller than the available area
        if content_lines > (area.height.saturating_sub(2)) as usize {
            let vertical_scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .symbols(scrollbar::VERTICAL)
                .begin_symbol(None)
                .end_symbol(None);

            vertical_scrollbar.render(vertical_scrollbar_area, buf, &mut vertical_scrollbar_state);
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
                &mut horizontal_scrollbar_state,
            );
        }
    }
}

impl Default for MessageLog {
    fn default() -> Self {
        Self::new()
    }
}
