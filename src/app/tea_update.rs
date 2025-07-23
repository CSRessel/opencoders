use crate::app::{
    event_msg::{Cmd, Msg},
    tea_model::{AppState, Model},
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
            model.clear_input_state();
            (model, Cmd::None)
        }

        Msg::ChangeState(new_state) => {
            model.state = new_state.clone();
            if matches!(model.state, AppState::Welcome) {
                model.clear_input_state();
            } else if matches!(model.state, AppState::TextEntry) {
                // Auto-scroll to bottom when entering text entry mode
                model.message_log.scroll_to_bottom();

                // If entering text entry mode, ensure we have a client and session
                if !model.is_session_ready() {
                    model.transition_to_connecting();
                    return (model, Cmd::AsyncSpawnClientDiscovery);
                }
            }
            (model, Cmd::None)
        }

        // Client initialization messages
        Msg::InitializeClient => {
            model.transition_to_connecting();
            (model, Cmd::AsyncSpawnClientDiscovery)
        }

        Msg::ClientConnected(client) => {
            model.client = Some(client.clone());
            model.transition_to_connected();
            (model, Cmd::AsyncSpawnSessionInit(client))
        }

        Msg::ClientConnectionFailed(error) => {
            let error_msg = format!("Failed to connect to OpenCode server: {}", error);
            model.transition_to_error(error_msg);
            (model, Cmd::None)
        }

        // Session management messages
        Msg::InitializeSession => {
            if let Some(client) = model.client.clone() {
                model.transition_to_connected();
                (model, Cmd::AsyncSpawnSessionInit(client))
            } else {
                // No client available, need to connect first
                model.transition_to_connecting();
                (model, Cmd::AsyncSpawnClientDiscovery)
            }
        }

        Msg::SessionReady(session) => {
            model.transition_to_session_ready(session);
            (model, Cmd::None)
        }

        Msg::SessionInitializationFailed(error) => {
            let error_msg = format!("Failed to initialize session: {}", error);
            model.transition_to_error(error_msg);
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

        // Task lifecycle messages
        Msg::TaskStarted(_task_id, _description) => {
            // Could update UI to show active tasks
            (model, Cmd::None)
        }

        Msg::TaskCompleted(_task_id) => {
            // Could update UI to remove completed task indicator
            (model, Cmd::None)
        }

        Msg::TaskFailed(_task_id, _error) => {
            // Could show error message or update connection status
            (model, Cmd::None)
        }

        // Progress reporting messages
        Msg::ConnectionProgress(_progress) => {
            // Could update a progress bar in UI
            (model, Cmd::None)
        }

        Msg::SessionProgress(_progress) => {
            // Could update a progress bar in UI
            (model, Cmd::None)
        }

        Msg::MarkMessagesViewed => {
            let count = model.messages_needing_stdout_print().len();
            model.mark_messages_printed_to_stdout(count);
            (model, Cmd::None)
        }
    }
}
