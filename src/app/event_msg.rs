#[derive(Debug, Clone, PartialEq)]
pub enum Msg {
    KeyPressed(char),
    Backspace,
    SubmitInput,
    ClearInput,
    ChangeState(crate::app::tea_model::AppState),
    Quit,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Cmd {
    None,
    // Future: API calls, file operations, etc.
}

#[derive(Debug, Clone, PartialEq)]
pub enum Sub {
    KeyboardInput,
}

