use crate::app::{
    event_msg::{Cmd, Msg},
    tea_model::{AppState, ConnectionStatus, Model},
    ui_components::text_input::TextInputEvent,
};
use opencode_sdk::models::{
    GetSessionByIdMessage200ResponseInner, Message, Part, TextPart, UserMessage,
};

pub fn update(mut model: Model, msg: Msg) -> (Model, Cmd) {
    match msg {
        Msg::KeyPressed(c) => {
            if let Some(submitted_text) = model.text_input.handle_event(TextInputEvent::Insert(c)) {
                model.last_input = Some(submitted_text);
            }
            (model, Cmd::None)
        }

        Msg::Backspace => {
            model.text_input.handle_event(TextInputEvent::Delete);
            (model, Cmd::None)
        }

        Msg::SubmitInput => {
            if let Some(submitted_text) = model.text_input.handle_event(TextInputEvent::Submit) {
                model.input_history.push(submitted_text.clone());
                model.last_input = Some(submitted_text.clone());

                model
                    .message_log
                    .create_and_push_user_message(&submitted_text)
            }
            (model, Cmd::None)
        }

        Msg::ClearInput => {
            model.text_input.clear();
            model.last_input = None;
            model.input_history.clear();
            model.printed_to_stdout_count = 0;
            (model, Cmd::None)
        }

        Msg::ChangeState(new_state) => {
            model.state = new_state.clone();
            if matches!(model.state, AppState::Welcome) {
                model.text_input.clear();
                model.last_input = None;
                model.input_history.clear();
                model.printed_to_stdout_count = 0;
            } else if matches!(model.state, AppState::TextEntry) {
                // Auto-scroll to bottom when entering text entry mode
                model.message_log.scroll_to_bottom();
                
                // If entering text entry mode, ensure we have a client and session
                if !model.is_session_ready() {
                    model.state = AppState::ConnectingToServer;
                    model.connection_status = ConnectionStatus::Connecting;
                    return (model, Cmd::DiscoverAndConnectClient);
                }
            }
            (model, Cmd::None)
        }

        // Client initialization messages
        Msg::InitializeClient => {
            model.state = AppState::ConnectingToServer;
            model.connection_status = ConnectionStatus::Connecting;
            (model, Cmd::DiscoverAndConnectClient)
        }

        Msg::ClientConnected(client) => {
            model.client = Some(client.clone());
            model.connection_status = ConnectionStatus::Connected;
            model.state = AppState::InitializingSession;
            (model, Cmd::InitializeSessionForClient(client))
        }

        Msg::ClientConnectionFailed(error) => {
            let error_msg = format!("Failed to connect to OpenCode server: {}", error);
            model.connection_status = ConnectionStatus::Error(error_msg.clone());
            model.state = AppState::ConnectionError(error_msg);
            (model, Cmd::None)
        }

        // Session management messages
        Msg::InitializeSession => {
            if let Some(client) = model.client.clone() {
                model.connection_status = ConnectionStatus::InitializingSession;
                model.state = AppState::InitializingSession;
                (model, Cmd::InitializeSessionForClient(client))
            } else {
                // No client available, need to connect first
                model.state = AppState::ConnectingToServer;
                model.connection_status = ConnectionStatus::Connecting;
                (model, Cmd::DiscoverAndConnectClient)
            }
        }

        Msg::SessionReady(session) => {
            model.session = Some(session);
            model.connection_status = ConnectionStatus::SessionReady;
            model.state = AppState::TextEntry;
            // Auto-scroll to bottom when session is ready
            model.message_log.scroll_to_bottom();
            (model, Cmd::None)
        }

        Msg::SessionInitializationFailed(error) => {
            let error_msg = format!("Failed to initialize session: {}", error);
            model.connection_status = ConnectionStatus::Error(error_msg.clone());
            model.state = AppState::ConnectionError(error_msg);
            (model, Cmd::None)
        }

        Msg::ChangeInline => {
            let new_inline = !model.init.inline_mode().clone();
            (model, Cmd::RebootTerminalWithInline(new_inline))
        }

        Msg::Quit => {
            model.state = AppState::Quit;
            (model, Cmd::None)
        }
        Msg::ScrollMessageLog(direction) => {
            model.message_log.move_message_log_scroll(&direction);
            (model, Cmd::None)
        }
        Msg::ScrollMessageLogHorizontal(direction) => {
            model.message_log.scroll_horizontal(direction);
            (model, Cmd::None)
        }
    }
}
