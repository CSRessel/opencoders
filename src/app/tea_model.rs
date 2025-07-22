use crate::app::ui_components::{MessageLog, TextInput};

#[derive(Debug, Clone, PartialEq)]
pub struct Model {
    pub init: ModelInit,
    // App state
    pub state: AppState,
    pub input_history: Vec<String>,
    pub last_input: Option<String>,
    pub printed_to_stdout_count: usize,
    // Stateful components:
    pub message_log: MessageLog,
    pub text_input: TextInput,
}

mod model_init {
    #[derive(Debug, Clone, PartialEq)]
    pub struct ModelInit {
        // Immutable initialization properties
        // that can't be changed without restarting the terminal
        init_height: u16,
        init_inline_mode: bool,
    }

    impl ModelInit {
        pub fn height(&self) -> u16 {
            self.init_height
        }

        pub fn inline_mode(&self) -> bool {
            self.init_inline_mode
        }
        pub fn new(height: u16, inline_mode: bool) -> ModelInit {
            ModelInit {
                init_height: height,
                init_inline_mode: inline_mode,
            }
        }
    }
}

pub use model_init::ModelInit;

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
            init: ModelInit::new(5, false),
            state: AppState::Welcome,
            input_history: Vec::new(),
            last_input: None,
            printed_to_stdout_count: 0,
            message_log,
            text_input,
        }
    }

    pub fn needs_manual_output(&self) -> bool {
        return self.init.inline_mode() & (self.messages_needing_stdout_print().len() > 0);
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
