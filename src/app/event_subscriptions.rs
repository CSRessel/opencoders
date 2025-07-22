use crate::app::{
    event_msg::{Msg, Sub},
    tea_model::{AppState, Model},
};
use crossterm::event::{self, Event, KeyCode, KeyModifiers};

pub fn subscriptions(model: &Model) -> Vec<Sub> {
    match model.state {
        AppState::Welcome | AppState::TextEntry => vec![Sub::KeyboardInput],
        AppState::Quit => vec![],
    }
}

pub fn poll_subscriptions(model: &Model) -> Result<Option<Msg>, Box<dyn std::error::Error>> {
    let subs = subscriptions(model);

    if subs.contains(&Sub::KeyboardInput) {
        if event::poll(std::time::Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                return Ok(crossterm_to_msg(key, model));
            }
        }
    }

    Ok(None)
}

fn crossterm_to_msg(key: crossterm::event::KeyEvent, model: &Model) -> Option<Msg> {
    match (&model.state, key.code, key.modifiers) {
        // Global quit commands
        (_, KeyCode::Char('q'), KeyModifiers::CONTROL) => Some(Msg::Quit),
        (AppState::Welcome, KeyCode::Char('q'), _) => Some(Msg::Quit),
        (AppState::Welcome, KeyCode::Esc, _) => Some(Msg::Quit),

        // State transitions
        (AppState::Welcome, KeyCode::Enter, _) => Some(Msg::ChangeState(AppState::TextEntry)),
        (AppState::TextEntry, KeyCode::Esc, _) => Some(Msg::ChangeState(AppState::Welcome)),

        // Text input events
        (AppState::TextEntry, KeyCode::Char(c), _) => Some(Msg::KeyPressed(c)),
        (AppState::TextEntry, KeyCode::Backspace, _) => Some(Msg::Backspace),
        (AppState::TextEntry, KeyCode::Enter, _) => Some(Msg::SubmitInput),

        // Message log scrolling
        (AppState::TextEntry, KeyCode::PageUp, _) => Some(Msg::ScrollMessageLog(-1)),
        (AppState::TextEntry, KeyCode::PageDown, _) => Some(Msg::ScrollMessageLog(1)),

        _ => None,
    }
}

