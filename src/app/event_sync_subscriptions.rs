use crate::app::{
    event_msg::{Msg, Sub},
    tea_model::{AppModalState, ConnectionStatus, EventStreamState, Model, RepeatShortcutKey},
    ui_components::{MsgTextArea, PopoverSelectorEvent},
};
use crossterm::event::{self, Event, KeyCode, KeyModifiers, MouseEventKind};

pub fn subscriptions(model: &Model) -> Vec<Sub> {
    let mut subs = match model.state {
        AppModalState::Quit => vec![],
        _ => vec![Sub::KeyboardInput, Sub::TerminalResize],
    };

    // Add event stream subscription when connected and in active states
    if model.is_session_ready()
        && matches!(model.event_stream_state, EventStreamState::Connected(_))
    {
        subs.push(Sub::EventStream);
    }

    subs
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
                (_, KeyCode::Char('l'), _, true) => Some(Msg::LeaderShowSessionSelector),
                (_, KeyCode::Tab, _, true) => Some(Msg::LeaderChangeInline),
                (_, KeyCode::Char('q'), _, true) => Some(Msg::Quit),

                (AppModalState::None, KeyCode::Char('c'), KeyModifiers::CONTROL, _) => {
                    if model.is_repeat_shortcut_timeout_active(RepeatShortcutKey::CtrlC) {
                        Some(Msg::Quit)
                    } else {
                        Some(Msg::TextArea(MsgTextArea::Clear))
                    }
                }
                // (AppModalState::None, KeyCode::Esc, __, _) => {
                //     // Leave session for main screen
                //     if model.is_repeat_shortcut_timeout_active(RepeatShortcutKey::Esc) {
                //         Some(Msg::SessionAbort)
                //     } else {
                //         Some(Msg::RepeatShortcutPressed(RepeatShortcutKey::Esc))
                //     }
                // }
                (AppModalState::Help | AppModalState::SelectSession, KeyCode::Esc, _, __) => {
                    // Close modals
                    Some(Msg::ChangeState(AppModalState::None))
                }
                (AppModalState::None, KeyCode::Char('r'), KeyModifiers::CONTROL, _) => {
                    Some(Msg::ToggleVerbosity)
                }
                (AppModalState::None, KeyCode::Enter, modifiers, _) => {
                    if modifiers.contains(KeyModifiers::SHIFT) {
                        Some(Msg::TextArea(MsgTextArea::Newline))
                    } else {
                        Some(Msg::SubmitTextInput)
                    }
                }
                (AppModalState::None, KeyCode::Tab, _, _) => Some(Msg::CycleModeState),

                // Message log scrolling (keeping Page Up/Down for message history)
                (AppModalState::None, KeyCode::PageUp, _, _) => Some(Msg::ScrollMessageLog(-5)),
                (AppModalState::None, KeyCode::PageDown, _, _) => Some(Msg::ScrollMessageLog(5)),

                // Session selector events
                (AppModalState::SelectSession, KeyCode::Up, _, _) => {
                    Some(Msg::SessionSelectorEvent(PopoverSelectorEvent::Up))
                }
                (AppModalState::SelectSession, KeyCode::Down, _, _)
                | (AppModalState::SelectSession, KeyCode::Tab, _, _) => {
                    Some(Msg::SessionSelectorEvent(PopoverSelectorEvent::Down))
                }
                (AppModalState::SelectSession, KeyCode::Char('k'), _, _) => {
                    Some(Msg::SessionSelectorEvent(PopoverSelectorEvent::Up))
                }
                (AppModalState::SelectSession, KeyCode::Char('j'), _, _) => {
                    Some(Msg::SessionSelectorEvent(PopoverSelectorEvent::Down))
                }
                (AppModalState::SelectSession, KeyCode::Enter, _, _) => {
                    Some(Msg::SessionSelectorEvent(PopoverSelectorEvent::Select))
                }
                (AppModalState::SelectSession, KeyCode::Esc, _, _) => {
                    Some(Msg::SessionSelectorEvent(PopoverSelectorEvent::Cancel))
                }

                // Text input events (internally routed to TextArea component for most keys)
                (AppModalState::None, _, _, _) => Some(Msg::TextArea(MsgTextArea::KeyInput(key))),

                // Retry connection
                (
                    AppModalState::Connecting(ConnectionStatus::Error(_)),
                    KeyCode::Char('r'),
                    _,
                    _,
                ) => Some(Msg::InitializeClient),
                (
                    AppModalState::Connecting(ConnectionStatus::Disconnected),
                    KeyCode::Char('r'),
                    _,
                    _,
                ) => {
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
            (AppModalState::None, MouseEventKind::ScrollUp) => Some(Msg::ScrollMessageLog(-1)),
            (AppModalState::None, MouseEventKind::ScrollDown) => Some(Msg::ScrollMessageLog(1)),
            _ => None,
        },
        Event::Resize(width, height) => Some(Msg::TerminalResize(width, height)),
        _ => None,
    }
}
