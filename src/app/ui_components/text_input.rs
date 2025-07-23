use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

#[derive(Debug, Clone, PartialEq)]
pub struct TextInput {
    content: String,
    cursor_position: usize,
    is_focused: bool,
    placeholder: String,
    session_id: Option<String>,
}

#[derive(Debug, Clone)]
pub enum TextInputEvent {
    Insert(char),
    Delete,
    Submit,
    Cancel,
}

impl TextInput {
    pub fn new() -> Self {
        Self {
            content: String::new(),
            cursor_position: 0,
            is_focused: false,
            placeholder: "Type your message...".to_string(),
            session_id: None,
        }
    }

    pub fn with_placeholder(placeholder: &str) -> Self {
        Self {
            content: String::new(),
            cursor_position: 0,
            is_focused: false,
            placeholder: placeholder.to_string(),
            session_id: None,
        }
    }

    pub fn handle_event(&mut self, event: TextInputEvent) -> Option<String> {
        match event {
            TextInputEvent::Insert(ch) => {
                if ch.is_control() {
                    return None;
                }
                let char_indices: Vec<_> = self.content.char_indices().collect();
                let cursor_char_index = self.cursor_position.min(char_indices.len());

                if cursor_char_index == char_indices.len() {
                    // Insert at end
                    self.content.push(ch);
                } else {
                    // Insert at specific position
                    let (byte_index, _) = char_indices[cursor_char_index];
                    self.content.insert(byte_index, ch);
                }
                self.cursor_position += 1;
                None
            }
            TextInputEvent::Delete => {
                if self.cursor_position > 0 && !self.content.is_empty() {
                    let char_indices: Vec<_> = self.content.char_indices().collect();
                    let cursor_char_index = self.cursor_position.min(char_indices.len());

                    if cursor_char_index > 0 {
                        let (byte_index, _) = char_indices[cursor_char_index - 1];
                        self.content.remove(byte_index);
                        self.cursor_position = cursor_char_index - 1;
                    }
                }
                None
            }
            TextInputEvent::Submit => {
                if !self.content.is_empty() {
                    let submitted_content = self.content.clone();
                    self.clear();
                    Some(submitted_content)
                } else {
                    None
                }
            }
            TextInputEvent::Cancel => {
                self.clear();
                None
            }
        }
    }

    pub fn set_focus(&mut self, focused: bool) {
        self.is_focused = focused;
    }

    pub fn is_focused(&self) -> bool {
        self.is_focused
    }

    pub fn clear(&mut self) {
        self.content.clear();
        self.cursor_position = 0;
    }

    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }

    pub fn set_session_id(&mut self, session_id: Option<String>) {
        self.session_id = session_id;
    }

    fn display_text(&self) -> String {
        if self.content.is_empty() && !self.is_focused {
            self.placeholder.clone()
        } else {
            let mut display = self.content.clone();
            if self.is_focused {
                let char_indices: Vec<_> = self.content.char_indices().collect();
                let cursor_char_index = self.cursor_position.min(char_indices.len());

                if cursor_char_index == char_indices.len() {
                    // Cursor at end
                    display.push('|');
                } else {
                    // Cursor in middle
                    let (byte_index, _) = char_indices[cursor_char_index];
                    display.insert(byte_index, '|');
                }
            }
            display
        }
    }
}

impl Widget for &TextInput {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let display_text = self.display_text();
        let is_placeholder = self.content.is_empty() && !self.is_focused;

        let style = if is_placeholder {
            Style::default().fg(Color::DarkGray)
        } else {
            Style::default().fg(Color::White)
        };

        // Split the area to accommodate status line if session ID exists
        let (input_area, status_area) = if self.session_id.is_some() {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3), // Input area (minimum 3 lines for borders + content)
                    Constraint::Length(1), // Status line
                ])
                .split(area);
            (chunks[0], Some(chunks[1]))
        } else {
            (area, None)
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(if self.is_focused {
                Style::default().fg(Color::Blue)
            } else {
                Style::default().fg(Color::Gray)
            });

        let paragraph = Paragraph::new(Line::from(Span::styled(display_text, style))).block(block);

        paragraph.render(input_area, buf);

        // Render session ID status line if present
        if let (Some(session_id), Some(status_area)) = (&self.session_id, status_area) {
            let status_text = format!("Session: {}", session_id);
            let status_paragraph = Paragraph::new(Line::from(Span::styled(
                status_text,
                Style::default().fg(Color::DarkGray),
            )));
            status_paragraph.render(status_area, buf);
        }
    }
}

impl Default for TextInput {
    fn default() -> Self {
        Self::new()
    }
}

