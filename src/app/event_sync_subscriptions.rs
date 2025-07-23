use crate::app::{
    event_msg::{Msg, Sub},
    tea_model::{AppState, Model},
};
use crossterm::event::{self, Event, KeyCode, KeyModifiers, MouseEventKind};

pub fn subscriptions(model: &Model) -> Vec<Sub> {
    match model.state {
        AppState::Welcome
        | AppState::TextEntry
        | AppState::ConnectingToServer
        | AppState::InitializingSession
        | AppState::ConnectionError(_) => vec![Sub::KeyboardInput],
        AppState::Quit => vec![],
    }
}

pub fn poll_subscriptions(model: &Model) -> Result<Option<Msg>, Box<dyn std::error::Error>> {
    let subs = subscriptions(model);

    if subs.contains(&Sub::KeyboardInput) {
        if event::poll(std::time::Duration::from_millis(16))? {
            return Ok(crossterm_to_msg(event::read()?, &model));
        }
    }

    Ok(None)
}

fn crossterm_to_msg(event: Event, model: &Model) -> Option<Msg> {
    match event {
        Event::Key(key) => {
            match (&model.state, key.code, key.modifiers) {
                // Global quit commands
                (_, KeyCode::Char('q'), KeyModifiers::CONTROL) => Some(Msg::Quit),
                (AppState::Welcome, KeyCode::Char('q'), _) => Some(Msg::Quit),
                (AppState::Welcome, KeyCode::Esc, _) => Some(Msg::Quit),
                (AppState::ConnectingToServer, KeyCode::Char('q'), _) => Some(Msg::Quit),
                (AppState::ConnectingToServer, KeyCode::Esc, _) => Some(Msg::Quit),
                (AppState::InitializingSession, KeyCode::Char('q'), _) => Some(Msg::Quit),
                (AppState::InitializingSession, KeyCode::Esc, _) => Some(Msg::Quit),
                (AppState::ConnectionError(_), KeyCode::Char('q'), _) => Some(Msg::Quit),
                (AppState::ConnectionError(_), KeyCode::Esc, _) => Some(Msg::Quit),

                // State transitions
                (AppState::Welcome, KeyCode::Enter, _) => {
                    Some(Msg::ChangeState(AppState::TextEntry))
                }
                (AppState::TextEntry, KeyCode::Esc, _) => Some(Msg::ChangeState(AppState::Welcome)),

                // Settings toggle
                (AppState::Welcome, KeyCode::Tab, _) => Some(Msg::ChangeInline),

                // Text input events
                (AppState::TextEntry, KeyCode::Char(c), _) => Some(Msg::KeyPressed(c)),
                (AppState::TextEntry, KeyCode::Backspace, _) => Some(Msg::Backspace),
                (AppState::TextEntry, KeyCode::Enter, _) => Some(Msg::SubmitInput),

                // Message log scrolling
                (AppState::TextEntry, KeyCode::PageUp, _) => Some(Msg::ScrollMessageLog(-10)),
                (AppState::TextEntry, KeyCode::PageDown, _) => Some(Msg::ScrollMessageLog(10)),
                (AppState::TextEntry, KeyCode::Up, _) => Some(Msg::ScrollMessageLog(-10)),
                (AppState::TextEntry, KeyCode::Down, _) => Some(Msg::ScrollMessageLog(10)),
                (AppState::TextEntry, KeyCode::Left, _) => {
                    Some(Msg::ScrollMessageLogHorizontal(-10))
                }
                (AppState::TextEntry, KeyCode::Right, _) => {
                    Some(Msg::ScrollMessageLogHorizontal(10))
                }

                // Retry connection
                (AppState::ConnectionError(_), KeyCode::Char('r'), _) => {
                    Some(Msg::InitializeClient)
                }

                _ => None,
            }
        }
        Event::Mouse(mouse) => match (&model.state, mouse.kind) {
            (AppState::TextEntry, MouseEventKind::ScrollUp) => Some(Msg::ScrollMessageLog(-1)),
            (AppState::TextEntry, MouseEventKind::ScrollDown) => Some(Msg::ScrollMessageLog(1)),
            _ => None,
        },
        _ => None,
    }
}
