use crate::app::event_msg::{Cmd, CmdOrBatch};
use crate::app::tea_model::{Model, RepeatShortcutKey, SessionState, INLINE_HEIGHT};
use crate::app::ui_components::{Block, Component, Paragraph};
use crate::app::view_model_context::ViewModelContext;
use crate::sdk::client::{generate_id, IdPrefix};
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
    HeightChanged(u16),
    FocusChanged(bool),
}

#[derive(Debug, Clone)]
pub struct TextInputArea {
    textarea: TextArea<'static>,
    min_height: u16,
    max_height: u16,
    current_height: u16,
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

// Helper function to convert CmdTextArea to main Cmd
pub fn handle_textarea_commands(model: &mut Model, commands: Vec<CmdTextArea>) -> CmdOrBatch {
    let mut main_commands = vec![];

    for command in commands {
        match command {
            CmdTextArea::Submit(text) => {
                // Handle text submission like the legacy SubmitInput logic
                model.input_history.push(text.clone());
                model.last_input = Some(text.clone());

                // If we have a pending session, create it now with this message
                if let SessionState::Pending(pending_info) = &model.session_state {
                    if let Some(client) = model.client.clone() {
                        model.session_state = SessionState::Creating(pending_info.clone());
                        model.pending_first_message = Some(text.clone());
                        model.session_is_idle = false;
                        main_commands.push(Cmd::AsyncCreateSessionWithMessage(client, text));
                        continue;
                    }
                }

                // If we have a ready session, send the message via API
                if let (Some(client), Some(session)) = (model.client.clone(), model.session()) {
                    let session_id = session.id.clone();
                    let (provider_id, model_id, mode) = model.get_mode_and_model_settings();
                    let message_id = generate_id(IdPrefix::Message);
                    model.session_is_idle = false;
                    main_commands.push(Cmd::AsyncSendUserMessage(
                        client,
                        session_id,
                        message_id,
                        text,
                        provider_id,
                        model_id,
                        mode,
                    ));
                }
            }
            CmdTextArea::HeightChanged(_height) => {
                // Handle height change if needed - currently no action required
            }
            CmdTextArea::FocusChanged(_focused) => {
                // Handle focus change if needed - currently no action required
            }
        }
    }

    CmdOrBatch::Batch(main_commands)
}

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

        // Render the textarea (no status bar logic here anymore)
        textarea.render(area, buf);
    }
}
