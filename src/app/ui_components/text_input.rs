use crate::app::event_msg::{Cmd, CmdOrBatch, Msg};
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

#[derive(Debug, Clone, PartialEq)]
pub enum MsgTextArea {
    KeyInput(KeyEvent),
    Newline,
    SetFocus(bool),
    Clear,
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
            current_height: TEXT_INPUT_AREA_MIN_HEIGHT,
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

    pub fn set_content(&mut self, content: &str) {
        self.textarea = TextArea::from(content.lines());
        self.textarea.set_cursor_line_style(Style::default());
        self.textarea.set_placeholder_text(&self.placeholder);
        let new_height = self.calculate_required_height();
        self.current_height = new_height;
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

    pub fn handle_input(&mut self, key_event: KeyEvent) -> InputResult {
        let old_height = self.current_height;

        // Filter out most newline input, except shift+enter
        let filtered_input = match (
            key_event.code,
            key_event.modifiers.contains(KeyModifiers::SHIFT),
        ) {
            (KeyCode::Enter, true) => {
                self.textarea.insert_newline();
                let new_height = self
                    .current_height
                    .saturating_add(1)
                    .min(TEXT_INPUT_AREA_MAX_HEIGHT);
                self.current_height = new_height;
                return InputResult {
                    submitted_text: None,
                    height_changed: self.current_height != old_height,
                    new_height: self.current_height,
                };
            }
            (KeyCode::Enter, false) => {
                // Should be handled before the event ever hits here
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
            (KeyCode::Char('@'), false) => {
                // Handle @ symbol - this will be processed by the parent update function
                Input::from(Event::Key(key_event))
            }
            _ => Input::from(Event::Key(key_event)),
        };
        // Disable alternative enter shorcuts
        // Input {
        //     key: Key::Char('m'),
        //     ctrl: true,
        //     ..
        // }
        // | Input {
        //     key: Key::Char('\n' | '\r'),
        //     ctrl: false,
        //     alt: false,
        //     ..
        // } => {
        //     return InputResult {
        //         submitted_text: None,
        //         height_changed: false,
        //         new_height: self.current_height,
        //     };
        // }
        //
        // And disable basically anything we don't need rn:
        // Input {
        //     key: Key::Tab,
        //     ctrl: false,
        //     alt: false,
        //     ..
        // }
        // | Input {
        //     key: Key::Char('h'),
        //     ctrl: true,
        //     alt: false,
        //     ..
        // } => {
        //     return InputResult {
        //         submitted_text: None,
        //         height_changed: false,
        //         new_height: self.current_height,
        //     }
        // }
        // Input {
        //     key: Key::Delete,
        //     ctrl: false,
        //     alt: true,
        //     ..
        // }
        // | Input {
        //     key: Key::Char('d'),
        //     ctrl: false,
        //     alt: true,
        //     ..
        // } => self.delete_next_word(),
        // Input {
        //     key: Key::Char('n'),
        //     ctrl: true,
        //     alt: false,
        //     shift,
        // }
        //
        // Input {
        //     key: Key::Char('p'),
        //     ctrl: true,
        //     alt: false,
        //     shift,
        // }
        //
        // But probably want:
        // Input {
        //     key: Key::Backspace,
        //     ctrl: true,
        //     alt: false,
        //     ..
        // } => {
        //     self.delete_word();
        //     return InputResult {
        //         submitted_text: None,
        //         height_changed: false,
        //         new_height: self.current_height,
        //     };
        // }
        // Input {
        //     key: Key::Down,
        //     ctrl: false,
        //     alt: false,
        //     shift,
        // } => {
        //     self.move_cursor_with_shift(CursorMove::Down, shift);
        //     false
        // }
        // Input {
        //     key: Key::Up,
        //     ctrl: false,
        //     alt: false,
        //     shift,
        // } => {
        //     self.move_cursor_with_shift(CursorMove::Up, shift);
        //     false
        // }
        //
        // Don't want
        // Input {
        //     key: Key::Char('f'),
        //     ctrl: true,
        //     alt: false,
        //     shift,
        // }
        // | Input {
        //     key: Key::Right,
        //     ctrl: false,
        //     alt: false,
        //     shift,
        // } => {
        //     self.move_cursor_with_shift(CursorMove::Forward, shift);
        //     false
        // }
        // Input {
        //     key: Key::Char('b'),
        //     ctrl: true,
        //     alt: false,
        //     shift,
        // }
        // | Input {
        //     key: Key::Left,
        //     ctrl: false,
        //     alt: false,
        //     shift,
        // } => {
        //     self.move_cursor_with_shift(CursorMove::Back, shift);
        //     false
        // }
        // Input {
        //     key: Key::Char('a'),
        //     ctrl: true,
        //     alt: false,
        //     shift,
        // }
        //
        // Probably want, at lease some of:
        // | Input {
        //     key: Key::Home,
        //     shift,
        //     ..
        // }
        // | Input {
        //     key: Key::Left | Key::Char('b'),
        //     ctrl: true,
        //     alt: true,
        //     shift,
        // } => {
        //     self.move_cursor_with_shift(CursorMove::Head, shift);
        //     false
        // }
        // Input {
        //     key: Key::Char('e'),
        //     ctrl: true,
        //     alt: false,
        //     shift,
        // }
        // | Input {
        //     key: Key::End,
        //     shift,
        //     ..
        // }
        // | Input {
        //     key: Key::Right | Key::Char('f'),
        //     ctrl: true,
        //     alt: true,
        //     shift,
        // } => {
        //     self.move_cursor_with_shift(CursorMove::End, shift);
        //     false
        // }
        // Input {
        //     key: Key::Char('<'),
        //     ctrl: false,
        //     alt: true,
        //     shift,
        // }
        // | Input {
        //     key: Key::Up | Key::Char('p'),
        //     ctrl: true,
        //     alt: true,
        //     shift,
        // } => {
        //     self.move_cursor_with_shift(CursorMove::Top, shift);
        //     false
        // }
        // Input {
        //     key: Key::Char('>'),
        //     ctrl: false,
        //     alt: true,
        //     shift,
        // }
        // | Input {
        //     key: Key::Down | Key::Char('n'),
        //     ctrl: true,
        //     alt: true,
        //     shift,
        // } => {
        //     self.move_cursor_with_shift(CursorMove::Bottom, shift);
        //     false
        // }
        // Input {
        //     key: Key::Char('f'),
        //     ctrl: false,
        //     alt: true,
        //     shift,
        // }
        // | Input {
        //     key: Key::Right,
        //     ctrl: true,
        //     alt: false,
        //     shift,
        // } => {
        //     self.move_cursor_with_shift(CursorMove::WordForward, shift);
        //     false
        // }
        // Input {
        //     key: Key::Char('b'),
        //     ctrl: false,
        //     alt: true,
        //     shift,
        // }
        // | Input {
        //     key: Key::Left,
        //     ctrl: true,
        //     alt: false,
        //     shift,
        // } => {
        //     self.move_cursor_with_shift(CursorMove::WordBack, shift);
        //     false
        // }
        // Input {
        //     key: Key::Char(']'),
        //     ctrl: false,
        //     alt: true,
        //     shift,
        // }
        // | Input {
        //     key: Key::Char('n'),
        //     ctrl: false,
        //     alt: true,
        //     shift,
        // }
        // | Input {
        //     key: Key::Down,
        //     ctrl: true,
        //     alt: false,
        //     shift,
        // } => {
        //     self.move_cursor_with_shift(CursorMove::ParagraphForward, shift);
        //     false
        // }
        // Input {
        //     key: Key::Char('['),
        //     ctrl: false,
        //     alt: true,
        //     shift,
        // }
        // | Input {
        //     key: Key::Char('p'),
        //     ctrl: false,
        //     alt: true,
        //     shift,
        // }
        // | Input {
        //     key: Key::Up,
        //     ctrl: true,
        //     alt: false,
        //     shift,
        // } => {
        //     self.move_cursor_with_shift(CursorMove::ParagraphBack, shift);
        //     false
        // }
        //
        // Need to replace somewhat:
        // Input {
        //     key: Key::Char('u'),
        //     ctrl: true,
        //     alt: false,
        //     ..
        // } => self.undo(),
        // Input {
        //     key: Key::Char('r'),
        //     ctrl: true,
        //     alt: false,
        //     ..
        // } => self.redo(),
        // Input {
        //     key: Key::Char('y'),
        //     ctrl: true,
        //     alt: false,
        //     ..
        // }
        // | Input {
        //     key: Key::Paste, ..
        // } => self.paste(),
        // Input {
        //     key: Key::Char('x'),
        //     ctrl: true,
        //     alt: false,
        //     ..
        // }
        // | Input { key: Key::Cut, .. } => self.cut(),
        // Input {
        //     key: Key::Char('c'),
        //     ctrl: true,
        //     alt: false,
        //     ..
        // }
        // | Input { key: Key::Copy, .. } => {
        //     self.copy();
        //     false
        // }
        // Input {
        //     key: Key::Char('v'),
        //     ctrl: true,
        //     alt: false,
        //     shift,
        // }
        // | Input {
        //     key: Key::PageDown,
        //     shift,
        //     ..
        // } => {
        //     self.scroll_with_shift(Scrolling::PageDown, shift);
        //     false
        // }
        // Input {
        //     key: Key::Char('v'),
        //     ctrl: false,
        //     alt: true,
        //     shift,
        // }
        // | Input {
        //     key: Key::PageUp,
        //     shift,
        //     ..
        // } => {
        //     self.scroll_with_shift(Scrolling::PageUp, shift);
        //     false
        // }
        // Input {
        //     key: Key::MouseScrollDown,
        //     shift,
        //     ..
        // } => {
        //     self.scroll_with_shift((1, 0).into(), shift);
        //     false
        // }
        // Input {
        //     key: Key::MouseScrollUp,
        //     shift,
        //     ..
        // } => {
        //     self.scroll_with_shift((-1, 0).into(), shift);
        //     false
        // }
        // _ => false,

        // TO SUPPORT:
        // Input {
        //     key: Key::Backspace,
        //     ctrl: true,
        //     alt: false,
        //     ..
        // } => {
        //     self.delete_word();
        //     return InputResult {
        //         submitted_text: None,
        //         height_changed: false,
        //         new_height: self.current_height,
        //     };
        // }
        // Input {
        //     key: Key::Down,
        //     ctrl: false,
        //     alt: false,
        //     shift,
        // } => {
        //     self.move_cursor_with_shift(CursorMove::Down, shift);
        //     false
        // }

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
    pub fn handle_message(&mut self, msg: MsgTextArea) {
        return match msg {
            MsgTextArea::Newline => {
                self.textarea.insert_newline();
                self.current_height = self.current_height.saturating_add(1);
            }
            MsgTextArea::KeyInput(key_event) => {
                self.handle_input(key_event);
            }
            MsgTextArea::SetFocus(focused) => {
                if self.is_focused != focused {
                    self.set_focus(focused);
                }
            }
            MsgTextArea::Clear => {
                self.clear();
            }
        };
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
