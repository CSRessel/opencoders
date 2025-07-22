use crate::app::ui_components::{MessageLog, TextInput};

#[derive(Debug, Clone, PartialEq)]
pub struct Model {
    // Init properties
    pub height: u16,
    pub inline_mode: bool,
    pub state: AppState,
    // App state
    pub input_history: Vec<String>,
    pub last_input: Option<String>,
    pub printed_to_stdout_count: usize,
    // Stateful components:
    pub message_log: MessageLog,
    pub text_input: TextInput,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AppState {
    Welcome,
    TextEntry,
    Quit,
}

impl Model {
    pub fn new() -> Self {
        let mut text_input = TextInput::new();
        text_input.set_focus(true);

        let message_log = MessageLog::new();

        Model {
            height: 5,
            inline_mode: false,
            state: AppState::Welcome,
            input_history: Vec::new(),
            last_input: None,
            printed_to_stdout_count: 0,
            message_log,
            text_input,
        }
    }

    pub fn needs_manual_output(&self) -> bool {
        return self.inline_mode & (self.messages_needing_stdout_print().len() > 0);
    }

    pub fn messages_needing_stdout_print(&self) -> &[String] {
        // All messages that haven't been printed to stdout yet
        let printed_count = self.printed_to_stdout_count;
        if printed_count < self.input_history.len() {
            &self.input_history[printed_count..]
        } else {
            &[]
        }
    }

    pub fn mark_messages_printed_to_stdout(&mut self, count: usize) {
        self.printed_to_stdout_count += count;
    }

    pub fn consume_viewed_state(&mut self) {
        self.mark_messages_printed_to_stdout(self.messages_needing_stdout_print().len());
    }
}
