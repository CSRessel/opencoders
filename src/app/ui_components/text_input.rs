use crate::app::tea_model::{Model, RepeatShortcutKey, INLINE_HEIGHT};
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
pub enum CmdTextArea {
    Submit(String),
    // HeightChanged(u16),
    // FocusChanged(bool),
}

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
pub const TEXT_INPUT_AREA_MAX_HEIGHT: u16 = INLINE_HEIGHT - 2; // configurable maximum

// E.g.:
// ╭─────────────────────────────────────────────────────────────────────────────────────────────╮
// │ >                                                                                           │
// ╰─────────────────────────────────────────────────────────────────────────────────────────────╯
// ⠧ Working                             Anthropic Claude Opus (21.4k tokens / 9% context) > build

impl TextInputArea {
    pub fn new() -> Self {
        let mut textarea = TextArea::default();
        textarea.set_cursor_line_style(Style::default()); // No cursor line highlighting by default

        Self {
            textarea,
            min_height: TEXT_INPUT_AREA_MIN_HEIGHT,
            max_height: TEXT_INPUT_AREA_MAX_HEIGHT,
            current_height: TEXT_INPUT_AREA_MAX_HEIGHT,
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
    pub fn handle_message(&mut self, msg: MsgTextArea) -> Vec<CmdTextArea> {
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
                            return vec![CmdTextArea::Submit(submitted_text)];
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
                    // commands.push(CmdTextArea::HeightChanged(self.current_height));
                }
                commands
            }
            MsgTextArea::SetFocus(focused) => {
                if self.is_focused != focused {
                    self.set_focus(focused);
                    vec![]
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
                    vec![CmdTextArea::Submit(submitted_text)]
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
