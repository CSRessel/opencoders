use crate::app::{
    event_msg::{Msg, Sub},
    tea_model::{AppState, ConnectionStatus, Model, RepeatShortcutKey},
    ui_components::PopoverSelectorEvent,
};
use crossterm::event::{self, Event, KeyCode, KeyModifiers, MouseEventKind};

pub fn subscriptions(model: &Model) -> Vec<Sub> {
    match model.state {
        AppState::Welcome
        | AppState::TextEntry
        | AppState::ConnectingToServer
        | AppState::InitializingSession
        | AppState::SelectSession
        | AppState::ConnectionError(_) => vec![Sub::KeyboardInput, Sub::TerminalResize],
        AppState::Quit => vec![],
    }
}

pub fn poll_subscriptions(model: &Model) -> Result<Option<Msg>, Box<dyn std::error::Error>> {
    let subs = subscriptions(model);

    if subs.contains(&Sub::KeyboardInput) || subs.contains(&Sub::TerminalResize) {
        if event::poll(std::time::Duration::from_millis(8))? {
            return Ok(crossterm_to_msg(event::read()?, &model));
        }
    }

    // Check for expired timeout and clear it
    if model.has_active_timeout() {
        if let Some(timeout) = &model.repeat_shortcut_timeout {
            if let Ok(elapsed) = timeout.started_at.elapsed() {
                if elapsed.as_millis() >= model.keys_shortcut_timeout_ms as u128 {
                    return Ok(Some(Msg::ClearTimeout));
                }
            }
        }
    }

    Ok(None)
}

pub fn crossterm_to_msg(event: Event, model: &Model) -> Option<Msg> {
    match event {
        Event::Key(key) => {
            match (&model.state, key.code, key.modifiers) {
                // Unified repeat shortcut timeout system
                (_, KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                    if model.is_repeat_shortcut_timeout_active(RepeatShortcutKey::CtrlC) {
                        Some(Msg::Quit)
                    } else {
                        Some(Msg::RepeatShortcutPressed(RepeatShortcutKey::CtrlC))
                    }
                }
                (_, KeyCode::Char('d'), KeyModifiers::CONTROL) => {
                    if model.is_repeat_shortcut_timeout_active(RepeatShortcutKey::CtrlD) {
                        Some(Msg::Quit)
                    } else {
                        Some(Msg::RepeatShortcutPressed(RepeatShortcutKey::CtrlD))
                    }
                }
                (AppState::TextEntry, KeyCode::Esc, __) => {
                    if model.is_repeat_shortcut_timeout_active(RepeatShortcutKey::Esc) {
                        Some(Msg::SessionAbort)
                    } else {
                        Some(Msg::RepeatShortcutPressed(RepeatShortcutKey::Esc))
                    }
                }

                // State transitions
                (AppState::Welcome, KeyCode::Enter, _) => {
                    Some(Msg::ChangeState(AppState::TextEntry))
                }
                (AppState::Welcome, KeyCode::Char('s'), _) => Some(Msg::ShowSessionSelector),

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

                // Session selector events
                (AppState::SelectSession, KeyCode::Up, _) => {
                    Some(Msg::SessionSelectorEvent(PopoverSelectorEvent::Up))
                }
                (AppState::SelectSession, KeyCode::Down, _) => {
                    Some(Msg::SessionSelectorEvent(PopoverSelectorEvent::Down))
                }
                (AppState::SelectSession, KeyCode::Char('k'), _) => {
                    Some(Msg::SessionSelectorEvent(PopoverSelectorEvent::Up))
                }
                (AppState::SelectSession, KeyCode::Char('j'), _) => {
                    Some(Msg::SessionSelectorEvent(PopoverSelectorEvent::Down))
                }
                (AppState::SelectSession, KeyCode::Enter, _) => {
                    Some(Msg::SessionSelectorEvent(PopoverSelectorEvent::Select))
                }
                (AppState::SelectSession, KeyCode::Esc, _) => {
                    Some(Msg::SessionSelectorEvent(PopoverSelectorEvent::Cancel))
                }

                // Retry connection
                (AppState::ConnectionError(_), KeyCode::Char('r'), _) => {
                    Some(Msg::InitializeClient)
                }
                (AppState::Welcome, KeyCode::Char('r'), _) => {
                    if matches!(model.connection_status, ConnectionStatus::Disconnected) {
                        Some(Msg::InitializeClient)
                    } else {
                        None
                    }
                }

                _ => {
                    // Clear timeout state when any other key is pressed
                    if model.has_active_timeout() {
                        Some(Msg::ClearTimeout)
                    } else {
                        None
                    }
                }
            }
        }
        Event::Mouse(mouse) => match (&model.state, mouse.kind) {
            (AppState::TextEntry, MouseEventKind::ScrollUp) => Some(Msg::ScrollMessageLog(-1)),
            (AppState::TextEntry, MouseEventKind::ScrollDown) => Some(Msg::ScrollMessageLog(1)),
            _ => None,
        },
        Event::Resize(width, height) => Some(Msg::TerminalResize(width, height)),
        _ => None,
    }
}
