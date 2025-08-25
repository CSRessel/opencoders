pub mod banner;
pub mod block;
pub mod inline_renderer;
pub mod message_log;
pub mod message_part;
pub mod paragraph;
pub mod popover_selector;
pub mod status_bar;
pub mod text_input;

pub use banner::create_welcome_text;
pub use block::Block;
pub use inline_renderer::render_text_inline;
pub use message_log::MessageLog;
pub use message_part::{MessageContext, MessagePart, MessageRenderer};
pub use paragraph::Paragraph;
pub use popover_selector::{PopoverSelector, PopoverSelectorEvent};
pub use status_bar::StatusBar;
pub use text_input::{CmdTextArea, InputResult, MsgTextArea, TextInputArea};

/// Component trait for modular ELM architecture
///
/// Components manage their own state and handle sub-messages, returning
/// commands that get translated to main application messages.
pub trait Component<State, SubMsg, SubCmd> {
    /// Handle a sub-message and update component state
    fn update(&mut self, msg: SubMsg, state: &State) -> Vec<SubCmd>;

    /// Get focus state for rendering
    fn is_focused(&self) -> bool;

    /// Set focus state
    fn set_focus(&mut self, focused: bool);
}

/// Behavioral traits for components

pub trait Focusable {
    fn is_focused(&self) -> bool;
    fn set_focus(&mut self, focused: bool);
}

pub trait DynamicSize {
    fn get_height(&self) -> u16;
    fn get_width(&self) -> u16;
}

pub trait Clearable {
    fn clear(&mut self);
}
