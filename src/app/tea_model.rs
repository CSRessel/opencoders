use crate::app::ui_components::TextInput;

#[derive(Debug, Clone, PartialEq)]
pub struct Model {
    pub state: AppState,
    pub text_input: TextInput,
    pub last_input: Option<String>,
    pub input_history: Vec<String>,
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
            input_history: Vec::new(),
        }
    }
}

