use crate::app::ui_components::TextInput;

#[derive(Debug, Clone, PartialEq)]
pub struct Model {
    pub printed_to_stdout_count: usize,
    pub text_input: TextInput,
    pub last_input: Option<String>,
    pub input_history: Vec<String>,
    pub state: AppState,
    pub inline_mode: bool,
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

        Model {
            state: AppState::Welcome,
            text_input,
            last_input: None,
            inline_mode: true,
            input_history: Vec::new(),
            printed_to_stdout_count: 0,
        }
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
}
