use crate::app::{
    event_msg::{Msg, Sub},
    tea_model::{AppModalState, ConnectionStatus, EventStreamState, Model, RepeatShortcutKey},
    ui_components::{MsgTextArea, MsgModalSessionSelector, MsgModalFileSelector, ModalSelectorEvent},
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
                // /new                      new session               ctrl+x n
                // /help                     show help                 ctrl+x h
                // /share                    share session             ctrl+x s
                // /models                   list models               ctrl+x m
                // /editor                   open editor               ctrl+x e
                // /init                     create/update AGENTS.md   ctrl+x i
                // /compact                  compact the session       ctrl+x c
                // /export                   export conversation       ctrl+x x
                // /sessions                 list sessions             ctrl+x l
                // /unshare                  unshare session           ctrl+x u
                // /themes                   list themes               ctrl+x t
                // /details                  toggle tool details       ctrl+x d
                // TODO the others, once those messages are supported
                (_, KeyCode::Char('l'), _, true) => Some(Msg::LeaderShowSessionSelector),
                (_, KeyCode::Tab, _, true) => Some(Msg::LeaderChangeInline),
                (_, KeyCode::Char('q'), _, true) => Some(Msg::Quit),

                // Works both without session (pending creation) and with explicit session
                (
                    AppModalState::None | AppModalState::Connecting(ConnectionStatus::Connected),
                    KeyCode::Enter,
                    modifiers,
                    _,
                ) => {
                    if modifiers.contains(KeyModifiers::SHIFT) {
                        Some(Msg::TextArea(MsgTextArea::Newline))
                    } else {
                        Some(Msg::SubmitTextInput)
                    }
                }
                (
                    AppModalState::None | AppModalState::Connecting(ConnectionStatus::Connected),
                    KeyCode::Tab,
                    _,
                    _,
                ) => Some(Msg::CycleModeState),
                (
                    AppModalState::None | AppModalState::Connecting(ConnectionStatus::Connected),
                    KeyCode::Char('c'),
                    KeyModifiers::CONTROL,
                    _,
                ) => {
                    if model.is_repeat_shortcut_timeout_active(RepeatShortcutKey::CtrlC) {
                        Some(Msg::Quit)
                    } else {
                        Some(Msg::TextArea(MsgTextArea::Clear))
                    }
                }

                // Requires session connected
                (AppModalState::None, KeyCode::Esc, __, _) => {
                    // Leave session for main screen
                    if model.is_repeat_shortcut_timeout_active(RepeatShortcutKey::Esc) {
                        Some(Msg::SessionAbort)
                    } else {
                        Some(Msg::RepeatShortcutPressed(RepeatShortcutKey::Esc))
                    }
                }
                (AppModalState::None, KeyCode::Char('r'), KeyModifiers::CONTROL, _) => {
                    Some(Msg::ToggleVerbosity)
                }
                // Message log scrolling (keeping Page Up/Down for fullscreen message history)
                (AppModalState::None, KeyCode::PageUp, _, _) => Some(Msg::ScrollMessageLog(-5)),
                (AppModalState::None, KeyCode::PageDown, _, _) => Some(Msg::ScrollMessageLog(5)),
                // Fall through for all other input
                (
                    AppModalState::None | AppModalState::Connecting(ConnectionStatus::Connected),
                    _,
                    _,
                    _,
                ) => Some(Msg::TextArea(MsgTextArea::KeyInput(key))),

                // Modal gated input handling
                (
                    AppModalState::ModalHelp | AppModalState::ModalSessionSelect,
                    KeyCode::Esc,
                    _,
                    __,
                ) => {
                    // Close modals
                    // TODO move to modal specific msg's
                    Some(Msg::ChangeState(AppModalState::None))
                }
                (AppModalState::ModalHelp, _, _, _) => None,
                // Session selector events
                (AppModalState::ModalSessionSelect, key_code, key_modifiers, _) => {
                    let key_event = crossterm::event::KeyEvent::new(key_code, key_modifiers);
                    Some(Msg::ModalSessionSelector(MsgModalSessionSelector::Event(
                        ModalSelectorEvent::KeyInput(key_event)
                    )))
                }
                (AppModalState::ModalSessionSelect, _, _, _) => None,
                // FileSelector events
                (AppModalState::ModalFileSelect, key_code, key_modifiers, _) => {
                    let key_event = crossterm::event::KeyEvent::new(key_code, key_modifiers);
                    Some(Msg::ModalFileSelector(MsgModalFileSelector::Event(
                        ModalSelectorEvent::KeyInput(key_event)
                    )))
                }
                (AppModalState::ModalFileSelect, _, _, _) => None,

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

                (
                    AppModalState::Connecting(ConnectionStatus::Disconnected)
                    | AppModalState::Connecting(ConnectionStatus::Error(_)),
                    KeyCode::Char('q'),
                    _,
                    _,
                ) => Some(Msg::Quit),
                (AppModalState::Quit, _, _, _) => Some(Msg::Quit),
                (AppModalState::Connecting(_), _, _, _) => None,
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
