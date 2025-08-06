use crate::app::{
    event_msg::{Msg, Sub},
    tea_model::{AppState, ConnectionStatus, EventStreamState, Model, RepeatShortcutKey},
    ui_components::PopoverSelectorEvent,
};
use crossterm::event::{self, Event, KeyCode, KeyModifiers, MouseEventKind};

pub fn subscriptions(model: &Model) -> Vec<Sub> {
    let mut subs = match model.state {
        AppState::Welcome
        | AppState::TextEntry
        | AppState::ConnectingToServer
        | AppState::InitializingSession
        | AppState::SelectSession
        | AppState::ConnectionError(_) => vec![Sub::KeyboardInput, Sub::TerminalResize],
        AppState::Quit => vec![],
    };

    // Add event stream subscription when connected and in active states
    if matches!(model.state, AppState::TextEntry | AppState::Welcome)
        && matches!(model.event_stream_state, EventStreamState::Connected(_))
    {
        subs.push(Sub::EventStream);
    }

    subs
}

pub fn poll_subscriptions(model: &Model) -> Result<Option<Msg>, Box<dyn std::error::Error>> {
    let subs = subscriptions(model);

    if subs.contains(&Sub::KeyboardInput) || subs.contains(&Sub::TerminalResize) {
        if event::poll(std::time::Duration::from_millis(8))? {
            return Ok(crossterm_to_msg(event::read()?, &model));
        }
    }

    // Poll event stream for new events
    if subs.contains(&Sub::EventStream) {
        if let EventStreamState::Connected(ref event_stream) = model.event_stream_state {
            // Clone the event stream handle to avoid borrowing issues
            let mut event_stream_clone = event_stream.clone();
            if let Some(event) = event_stream_clone.try_next_event() {
                return Ok(Some(Msg::EventReceived(event)));
            }
        }
    }

    // Check for expired timeout and clear it
    if model.has_active_timeout() {
        if let Some(timeout) = &model.repeat_shortcut_timeout {
            if let Ok(elapsed) = timeout.started_at.elapsed() {
                if elapsed.as_millis() >= model.config.keys_shortcut_timeout_ms as u128 {
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
            match (
                &model.state,
                key.code,
                key.modifiers,
                model.is_repeat_shortcut_timeout_active(RepeatShortcutKey::Leader),
            ) {
                // Unified repeat shortcut timeout system
                (_, KeyCode::Char('c'), KeyModifiers::CONTROL, _) => {
                    if model.is_repeat_shortcut_timeout_active(RepeatShortcutKey::CtrlC) {
                        Some(Msg::Quit)
                    } else {
                        Some(Msg::RepeatShortcutPressed(RepeatShortcutKey::CtrlC))
                    }
                }
                (_, KeyCode::Char('d'), KeyModifiers::CONTROL, _) => {
                    if model.is_repeat_shortcut_timeout_active(RepeatShortcutKey::CtrlD) {
                        Some(Msg::Quit)
                    } else {
                        Some(Msg::RepeatShortcutPressed(RepeatShortcutKey::CtrlD))
                    }
                }
                (AppState::TextEntry, KeyCode::Esc, __, _) => {
                    if model.is_repeat_shortcut_timeout_active(RepeatShortcutKey::Esc) {
                        Some(Msg::SessionAbort)
                    } else {
                        Some(Msg::RepeatShortcutPressed(RepeatShortcutKey::Esc))
                    }
                }
                (_, KeyCode::Char('x'), KeyModifiers::CONTROL, _) => {
                    Some(Msg::RepeatShortcutPressed(RepeatShortcutKey::Leader))
                }
                // Leader shortcuts:
                // /new                      new session               ctrl+x n                ┃
                // /help                     show help                 ctrl+x h                ┃
                // /share                    share session             ctrl+x s                ┃
                // /models                   list models               ctrl+x m                ┃
                // /editor                   open editor               ctrl+x e                ┃
                // /init                     create/update AGENTS.md   ctrl+x i                ┃
                // /compact                  compact the session       ctrl+x c                ┃
                // /export                   export conversation       ctrl+x x                ┃
                // /sessions                 list sessions             ctrl+x l                ┃
                // /unshare                  unshare session           ctrl+x u                ┃
                // /themes                   list themes               ctrl+x t                ┃
                // /details                  toggle tool details       ctrl+x d                ┃
                // TODO the others, once those messages are supported
                (_, KeyCode::Char('l'), _, true) => Some(Msg::ShowSessionSelector),
                (_, KeyCode::Tab, _, true) => Some(Msg::ChangeInline),
                (_, KeyCode::Char('q'), _, true) => Some(Msg::Quit),

                (AppState::Welcome, KeyCode::Enter, _, _) => {
                    Some(Msg::ChangeState(AppState::TextEntry))
                }

                // Text input events
                (AppState::TextEntry, KeyCode::Char(c), _, _) => Some(Msg::KeyPressed(c)),
                (AppState::TextEntry, KeyCode::Backspace, _, _) => Some(Msg::Backspace),
                (AppState::TextEntry, KeyCode::Enter, _, _) => Some(Msg::SubmitInput),
                (AppState::TextEntry, KeyCode::Tab, _, _) => Some(Msg::CycleModeState),

                // Message log scrolling
                (AppState::TextEntry, KeyCode::PageUp, _, _) => Some(Msg::ScrollMessageLog(-5)),
                (AppState::TextEntry, KeyCode::PageDown, _, _) => Some(Msg::ScrollMessageLog(5)),
                (AppState::TextEntry, KeyCode::Up, _, _) => Some(Msg::ScrollMessageLog(-5)),
                (AppState::TextEntry, KeyCode::Down, _, _) => Some(Msg::ScrollMessageLog(5)),
                (AppState::TextEntry, KeyCode::Left, _, _) => {
                    Some(Msg::ScrollMessageLogHorizontal(-5))
                }
                (AppState::TextEntry, KeyCode::Right, _, _) => {
                    Some(Msg::ScrollMessageLogHorizontal(5))
                }

                // Session selector events
                (AppState::SelectSession, KeyCode::Up, _, _) => {
                    Some(Msg::SessionSelectorEvent(PopoverSelectorEvent::Up))
                }
                (AppState::SelectSession, KeyCode::Down, _, _)
                | (AppState::SelectSession, KeyCode::Tab, _, _) => {
                    Some(Msg::SessionSelectorEvent(PopoverSelectorEvent::Down))
                }
                (AppState::SelectSession, KeyCode::Char('k'), _, _) => {
                    Some(Msg::SessionSelectorEvent(PopoverSelectorEvent::Up))
                }
                (AppState::SelectSession, KeyCode::Char('j'), _, _) => {
                    Some(Msg::SessionSelectorEvent(PopoverSelectorEvent::Down))
                }
                (AppState::SelectSession, KeyCode::Enter, _, _) => {
                    Some(Msg::SessionSelectorEvent(PopoverSelectorEvent::Select))
                }
                (AppState::SelectSession, KeyCode::Esc, _, _) => {
                    Some(Msg::SessionSelectorEvent(PopoverSelectorEvent::Cancel))
                }

                // Retry connection
                (AppState::ConnectionError(_), KeyCode::Char('r'), _, _) => {
                    Some(Msg::InitializeClient)
                }
                (AppState::Welcome, KeyCode::Char('r'), _, _) => {
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
