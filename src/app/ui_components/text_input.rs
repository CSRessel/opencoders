use crate::app::tea_model::{Model, RepeatShortcutKey};
use crate::app::ui_components::{Block, Component, Paragraph};
use crate::app::view_model_context::ViewModelContext;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Borders, Widget},
};
use throbber_widgets_tui::{Throbber, ThrobberState};
use tui_textarea::{Input, Key, TextArea};

const MODE_COLORS: [Color; 3] = [Color::Black, Color::Magenta, Color::Green];
const MODE_DEFAULT_COLOR: Color = Color::Gray;

// Message types for modular ELM architecture
#[derive(Debug, Clone, PartialEq)]
pub enum MsgTextArea {
    KeyInput(KeyEvent),
    SetFocus(bool),
    Clear,
    Submit,
}

#[derive(Debug, Clone)]
pub enum TextAreaCmd {
    Submit(String),
    HeightChanged(u16),
    FocusChanged(bool),
}

// Legacy TextInput struct - keeping for now during transition
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

// New TextInputArea struct wrapping tui_textarea::TextArea
#[derive(Debug, Clone)]
pub struct TextInputArea {
    textarea: TextArea<'static>,
    min_height: u16,
    max_height: u16,
    current_height: u16,
    session_id: Option<String>,
    placeholder: String,
    is_focused: bool,
}

// Result type for input handling
#[derive(Debug)]
pub struct InputResult {
    pub submitted_text: Option<String>,
    pub height_changed: bool,
    pub new_height: u16,
}

pub const TEXT_INPUT_HEIGHT: u16 = 4;
pub const TEXT_INPUT_AREA_MIN_HEIGHT: u16 = 3; // minimum: border + content + border
pub const TEXT_INPUT_AREA_MAX_HEIGHT: u16 = 10; // configurable maximum

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

    // Conversion function from crossterm KeyEvent to tui_textarea Input
    pub fn crossterm_key_to_textarea_input(key_event: KeyEvent) -> Input {
        let KeyEvent {
            code, modifiers, ..
        } = key_event;

        let key = match code {
            KeyCode::Char(c) => Key::Char(c),
            KeyCode::Enter => Key::Enter,
            KeyCode::Left => Key::Left,
            KeyCode::Right => Key::Right,
            KeyCode::Up => Key::Up,
            KeyCode::Down => Key::Down,
            KeyCode::Home => Key::Home,
            KeyCode::End => Key::End,
            KeyCode::PageUp => Key::PageUp,
            KeyCode::PageDown => Key::PageDown,
            KeyCode::Tab => Key::Tab,
            KeyCode::Delete => Key::Delete,
            KeyCode::F(f) => Key::F(f),
            KeyCode::Backspace => Key::Backspace,
            KeyCode::Esc => Key::Esc,
            _ => Key::Null, // Fallback for unmapped keys
        };

        Input {
            key,
            ctrl: modifiers.contains(KeyModifiers::CONTROL),
            alt: modifiers.contains(KeyModifiers::ALT),
            shift: modifiers.contains(KeyModifiers::SHIFT),
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
            let loading_label = match (
                &model.get().has_active_timeout(),
                &model.get().repeat_shortcut_timeout,
                &model.get().active_task_count,
            ) {
                (true, Some(timeout), _) => match timeout.key {
                    RepeatShortcutKey::Leader => "Shortcut waiting...",
                    RepeatShortcutKey::CtrlC => "Ctrl+C again to confirm",
                    RepeatShortcutKey::CtrlD => "Ctrl+D again to confirm",
                    RepeatShortcutKey::Esc => "Esc again to confirm",
                },
                (_, _, 0) => "Ready",
                _ => "Working...",
            };
            enum LoadingWidget<'a> {
                Throbber(Throbber<'a>),
                Paragraph(Paragraph<'a>),
            }

            impl<'a> Widget for LoadingWidget<'a> {
                fn render(self, area: Rect, buf: &mut Buffer) {
                    match self {
                        LoadingWidget::Throbber(t) => t.render(area, buf),
                        LoadingWidget::Paragraph(p) => p.render(area, buf),
                    }
                }
            }

            let loading_paragraph =
                if !model.get().session_is_idle || model.get().active_task_count > 0 {
                    LoadingWidget::Throbber(Throbber::default().label(loading_label))
                } else {
                    LoadingWidget::Paragraph(Paragraph::new(loading_label))
                };

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

// Implementation for TextInputArea
impl TextInputArea {
    pub fn new() -> Self {
        let mut textarea = TextArea::default();
        textarea.set_cursor_line_style(Style::default()); // No cursor line highlighting by default

        Self {
            textarea,
            min_height: TEXT_INPUT_AREA_MIN_HEIGHT,
            max_height: TEXT_INPUT_AREA_MAX_HEIGHT,
            current_height: TEXT_INPUT_AREA_MIN_HEIGHT,
            session_id: None,
            placeholder: "Type your message...".to_string(),
            is_focused: false,
        }
    }

    pub fn with_placeholder(placeholder: &str) -> Self {
        let mut instance = Self::new();
        instance.placeholder = placeholder.to_string();
        instance.textarea.set_placeholder_text(placeholder);
        instance
    }

    pub fn set_focus(&mut self, focused: bool) {
        self.is_focused = focused;
    }

    pub fn is_focused(&self) -> bool {
        self.is_focused
    }

    pub fn set_session_id(&mut self, session_id: Option<String>) {
        self.session_id = session_id;
    }

    pub fn clear(&mut self) {
        self.textarea = TextArea::default();
        self.textarea.set_cursor_line_style(Style::default());
        self.textarea.set_placeholder_text(&self.placeholder);
        self.current_height = self.min_height;
    }

    pub fn content(&self) -> String {
        self.textarea.lines().join("\n")
    }

    pub fn is_empty(&self) -> bool {
        self.textarea.lines().len() == 1 && self.textarea.lines()[0].is_empty()
    }

    fn calculate_required_height(&self) -> u16 {
        let content_lines = self.textarea.lines().len() as u16;
        let required = (content_lines + 2).max(self.min_height); // +2 for borders
        required.min(self.max_height)
    }

    pub fn current_height(&self) -> u16 {
        self.current_height
    }

    pub fn handle_input(&mut self, event: Event) -> InputResult {
        // self.textarea
        //     .input(TextArea::input(&mut self.textarea, event));

        let old_height = self.current_height;

        // Filter out Enter/newline input to maintain single-line behavior for now
        let filtered_input = match event {
            Event::Key(key_event) => match key_event.code {
                KeyCode::Enter => {
                    if !self.is_empty() {
                        let submitted_text = self.content();
                        self.clear();
                        return InputResult {
                            submitted_text: Some(submitted_text),
                            height_changed: self.current_height != old_height,
                            new_height: self.current_height,
                        };
                    } else {
                        return InputResult {
                            submitted_text: None,
                            height_changed: false,
                            new_height: self.current_height,
                        };
                    }
                }
                _ => Input::from(Event::Key(key_event)),
            },
            // Input {
            //     key: Key::Char('m'),
            //     ctrl: true,
            //     ..
            // } => {
            //     // Disable Ctrl+M (alternative Enter) to prevent newlines
            //     return InputResult {
            //         submitted_text: None,
            //         height_changed: false,
            //         new_height: self.current_height,
            //     };
            // }
            // Event::Key(key_event) => Input {
            //     key: key_event.code.key........,
            //     alt: false,
            //     ctrl: false,
            //     shift: false,
            // },
            other => Input::from(other),
        };

        // Process the input through textarea
        self.textarea.input(filtered_input);

        // Recalculate height
        self.current_height = self.calculate_required_height();

        InputResult {
            submitted_text: None,
            height_changed: self.current_height != old_height,
            new_height: self.current_height,
        }
    }
}

// Component trait implementation for TextInputArea
impl TextInputArea {
    pub fn handle_message(&mut self, msg: MsgTextArea) -> Vec<TextAreaCmd> {
        match msg {
            MsgTextArea::KeyInput(key_event) => {
                let event = Event::Key(key_event);
                let old_height = self.current_height;

                // Process the input similar to handle_input
                let filtered_input = match key_event.code {
                    KeyCode::Enter => {
                        if !self.is_empty() {
                            let submitted_text = self.content();
                            self.clear();
                            return vec![TextAreaCmd::Submit(submitted_text)];
                        } else {
                            return vec![];
                        }
                    }
                    _ => Input::from(event),
                };

                // Process the input through textarea
                self.textarea.input(filtered_input);

                // Recalculate height
                self.current_height = self.calculate_required_height();

                // Return commands for any changes
                let mut commands = vec![];
                if self.current_height != old_height {
                    commands.push(TextAreaCmd::HeightChanged(self.current_height));
                }
                commands
            }
            MsgTextArea::SetFocus(focused) => {
                if self.is_focused != focused {
                    self.set_focus(focused);
                    vec![TextAreaCmd::FocusChanged(focused)]
                } else {
                    vec![]
                }
            }
            MsgTextArea::Clear => {
                self.clear();
                vec![]
            }
            MsgTextArea::Submit => {
                if !self.is_empty() {
                    let submitted_text = self.content();
                    self.clear();
                    vec![TextAreaCmd::Submit(submitted_text)]
                } else {
                    vec![]
                }
            }
        }
    }
}

impl Default for TextInputArea {
    fn default() -> Self {
        Self::new()
    }
}

// Widget implementation for TextInputArea
impl Widget for &TextInputArea {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let model = ViewModelContext::current();

        // Split the area to accommodate status line if session ID exists
        let (input_area, status_area) = {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(self.current_height), // Dynamic input area height
                    Constraint::Length(1),                   // Status line
                ])
                .split(area);
            (chunks[0], Some(chunks[1]))
        };

        // Create a mutable textarea for rendering with proper styling
        let mut textarea = self.textarea.clone();

        // Set up the block with focus-dependent styling
        let block = ratatui::widgets::Block::default()
            .borders(Borders::ALL)
            .border_style(if self.is_focused {
                Style::default().fg(Color::Blue)
            } else {
                Style::default().fg(Color::Gray)
            });

        textarea.set_block(block);

        // Render the textarea
        textarea.render(input_area, buf);

        // Render status line (preserving existing logic from TextInput)
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
            let loading_label = match (
                &model.get().has_active_timeout(),
                &model.get().repeat_shortcut_timeout,
                &model.get().active_task_count,
            ) {
                (true, Some(timeout), _) => match timeout.key {
                    RepeatShortcutKey::Leader => "Shortcut waiting...",
                    RepeatShortcutKey::CtrlC => "Ctrl+C again to confirm",
                    RepeatShortcutKey::CtrlD => "Ctrl+D again to confirm",
                    RepeatShortcutKey::Esc => "Esc again to confirm",
                },
                (_, _, 0) => "Ready",
                _ => "Working...",
            };

            enum LoadingWidget<'a> {
                Throbber(Throbber<'a>),
                Paragraph(Paragraph<'a>),
            }

            impl<'a> Widget for LoadingWidget<'a> {
                fn render(self, area: Rect, buf: &mut Buffer) {
                    match self {
                        LoadingWidget::Throbber(t) => t.render(area, buf),
                        LoadingWidget::Paragraph(p) => p.render(area, buf),
                    }
                }
            }

            let loading_paragraph =
                if !model.get().session_is_idle || model.get().active_task_count > 0 {
                    LoadingWidget::Throbber(Throbber::default().label(loading_label))
                } else {
                    LoadingWidget::Paragraph(Paragraph::new(loading_label))
                };

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
