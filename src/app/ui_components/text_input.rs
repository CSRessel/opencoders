use crate::app::tea_model::RepeatShortcutKey;
use crate::app::ui_components::{Block, Paragraph};
use crate::app::view_model_context::ViewModelContext;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Borders, Widget},
};

const MODE_COLORS: [Color; 3] = [Color::Black, Color::Magenta, Color::Green];
const MODE_DEFAULT_COLOR: Color = Color::Gray;

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

pub const TEXT_INPUT_HEIGHT: u16 = 4;
// E.g.:
// ╭─────────────────────────────────────────────────────────────────────────────────────────────╮
// │ >                                                                                           │
// ╰─────────────────────────────────────────────────────────────────────────────────────────────╯
// ⠧ Working                             Anthropic Claude Opus (21.4k tokens / 9% context) > build

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

    fn prompt_start(&self) -> Span {
        Span::styled(" > ", Style::default().fg(Color::Gray))
    }

    fn display_text(&self) -> Vec<Span> {
        if self.content.is_empty() && !self.is_focused {
            vec![
                self.prompt_start(),
                Span::styled(self.placeholder.clone(), Style::default().fg(Color::Gray)),
            ]
        } else {
            let mut displayed = vec![self.prompt_start()];
            let text = self.content.clone();
            if self.is_focused {
                let char_indices: Vec<_> = self.content.char_indices().collect();
                let cursor_char_index = self.cursor_position.min(char_indices.len());
                let cursor_style = Style::default().fg(Color::Black).bg(Color::White);
                let text_style = Style::default().fg(Color::White);

                if cursor_char_index == char_indices.len() {
                    // Cursor at end
                    displayed.push(Span::styled(text, text_style));
                    displayed.push(Span::styled(" ", cursor_style));
                } else {
                    // Cursor in middle
                    let (byte_index, byte_char) = char_indices[cursor_char_index];
                    displayed.push(Span::styled(text[0..byte_index].to_string(), text_style));
                    displayed.push(Span::styled(byte_char.to_string(), cursor_style));
                    displayed.push(Span::styled(
                        text[byte_index..char_indices.len()].to_string(),
                        text_style,
                    ))
                }
            }
            displayed
        }
    }
}

impl Widget for &TextInput {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let model = ViewModelContext::current();
        let display_text = self.display_text();

        // Split the area to accommodate status line if session ID exists
        let (input_area, status_area) = {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3), // Input area (minimum 3 lines for borders + content)
                    Constraint::Length(1), // Status line
                ])
                .split(area);
            (chunks[0], Some(chunks[1]))
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(if self.is_focused {
                Style::default().fg(Color::Blue)
            } else {
                Style::default().fg(Color::Gray)
            });

        let paragraph = Paragraph::new(Line::from(display_text)).block(block);

        paragraph.render(input_area, buf);

        if let Some(status_area) = status_area {
            // Get current mode info for display
            let (mode_text, mode_color) = if let Some(mode_index) = model.get().mode_state {
                let bg_color = MODE_COLORS
                    .get(mode_index as usize)
                    .copied()
                    .unwrap_or(MODE_DEFAULT_COLOR);
                (
                    model
                        .get()
                        .get_current_mode_name()
                        .unwrap_or("UNKNOWN".to_string()),
                    bg_color,
                )
            } else {
                ("UNKNOWN".to_string(), MODE_DEFAULT_COLOR)
            };
            let mut mode_len = mode_text.len();
            let mode_padding = " ".repeat(8 - mode_len);
            mode_len += mode_padding.len();
            // Render mode with background color
            let mode_paragraph = Paragraph::new(Line::from(Span::styled(
                format!(" {}{} ", mode_text, mode_padding),
                Style::default().bg(mode_color).fg(Color::White),
            )));

            let status_text = format!(
                " {} {} (20.4k tokens / 9% context) >",
                model.get().sdk_provider,
                model.get().sdk_model,
            );
            let status_len = status_text.len();
            let status_paragraph = Paragraph::new(Line::from(status_text));

            // Check for active repeat shortcut timeout and show appropriate message
            let loading_label = if let Some(timeout) = &model.get().repeat_shortcut_timeout {
                // Check if timeout is still active, not just if it exists
                if model.get().has_active_timeout() {
                    match timeout.key {
                        RepeatShortcutKey::Leader => "Shortcut waiting...",
                        RepeatShortcutKey::CtrlC => "Ctrl+C again to confirm",
                        RepeatShortcutKey::CtrlD => "Ctrl+D again to confirm",
                        RepeatShortcutKey::Esc => "Esc again to confirm",
                    }
                } else {
                    "Working..."
                }
            } else {
                "Working..."
            };
            let loading_paragraph = throbber_widgets_tui::Throbber::default().label(loading_label);

            let (status_line_start, status_line_center, status_line_provider, status_line_mode) = {
                let start_width = (area.width / 4).min(10);
                let chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Min(start_width / 2),
                        Constraint::Min(start_width),
                        Constraint::Length(status_len as u16),
                        Constraint::Length(mode_len as u16),
                    ])
                    .split(status_area);
                (chunks[0], chunks[1], chunks[2], chunks[3])
            };

            loading_paragraph.render(status_line_start, buf);

            // Render session ID status line if present
            if let Some(session_id) = &self.session_id {
                let session_paragraph = Paragraph::new(Line::from(Span::styled(
                    session_id,
                    Style::default().fg(Color::DarkGray),
                )));
                session_paragraph.render(status_line_center, buf);
            }

            status_paragraph.render(status_line_provider, buf);

            mode_paragraph.render(status_line_mode, buf);
        }
    }
}

impl Default for TextInput {
    fn default() -> Self {
        Self::new()
    }
}
