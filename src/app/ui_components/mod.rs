pub mod banner;
pub mod block;
pub mod message_log;
pub mod message_part;
pub mod modal_file_selector;
pub mod modal_selector;
pub mod modal_session_selector;
pub mod paragraph;
pub mod status_bar;
pub mod text_input;

pub use banner::create_welcome_text;
pub use block::Block;
pub use message_log::MessageLog;
pub use message_part::{MessageContext, MessagePart, MessageRenderer};
pub use modal_file_selector::FileSelector;
pub use modal_selector::{ModalSelector, SelectableData, SelectorConfig, SelectorMode, TableColumn};
pub use modal_session_selector::{SessionSelector, SessionEvent};
pub use paragraph::Paragraph;
pub use status_bar::StatusBar;
pub use text_input::{InputResult, MsgTextArea, TextInputArea};

use crate::app::event_msg::CmdOrBatch;

/// Component trait for modular ELM architecture
///
/// Components manage their own state and handle sub-messages, returning
/// commands that get translated to main application messages.
pub trait Component<State, SubMsg, SubCmd> {
    /// Handle a sub-message and update component state
    fn update(&mut self, msg: SubMsg, state: &State) -> CmdOrBatch<SubCmd>;

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
