use crate::app::{
    event_msg::*,
    tea_model::*,
    ui_components::{text_input::TextInputEvent, PopoverSelectorEvent},
};
use opencode_sdk::models::{
    GetSessionByIdMessage200ResponseInner, Message, Part, TextPart, UserMessage,
};
use std::time::SystemTime;

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

                // If we have a pending session, create it now with this message
                if let SessionState::Pending(pending_info) = &model.session_state {
                    if let Some(client) = model.client.clone() {
                        model.session_state = SessionState::Creating(pending_info.clone());
                        model.pending_first_message = Some(submitted_text.clone());
                        return (
                            model,
                            Cmd::AsyncCreateSessionWithMessage(client, submitted_text),
                        );
                    }
                }

                model
                    .message_log
                    .create_and_push_user_message(&submitted_text);
            }
            (model, Cmd::None)
        }

        Msg::ClearInput => {
            model.clear_input_state();
            (model, Cmd::None)
        }

        Msg::ChangeState(new_state) => {
            // If trying to enter TextEntry but session isn't ready, trigger session init
            if matches!(new_state, AppState::TextEntry) && !model.is_session_ready() {
                // Same as selecting the "Create New" option
                model.change_session(Some(0));
                return (model, Cmd::None);
            }

            model.state = new_state.clone();
            if matches!(model.state, AppState::Welcome) {
                model.clear_input_state();
            } else if matches!(model.state, AppState::TextEntry) {
                // Auto-scroll to bottom when entering text entry mode
                model.message_log.touch_scroll();
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
            (model, Cmd::None)
        }

        Msg::ClientConnectionFailed(error) => {
            let error_msg = format!("Failed to connect to OpenCode server: {}", error);
            model.transition_to_error(error_msg);
            (model, Cmd::None)
        }

        // Session management messages
        Msg::SessionReady(session) => {
            let session_id = session.id.clone();
            model.state = AppState::TextEntry;

            // Set session data
            model.text_input.set_session_id(Some(session.id.clone()));
            model.session_state = SessionState::Ready(session);
            model.connection_status = ConnectionStatus::SessionReady;
            model.message_log.touch_scroll();

            // Fetch session messages once session is ready
            if let Some(client) = model.client.clone() {
                (model, Cmd::AsyncLoadSessionMessages(client, session_id))
            } else {
                (model, Cmd::None)
            }
        }

        Msg::SessionCreatedWithMessage(session, first_message) => {
            let session_id = session.id.clone();
            model.state = AppState::TextEntry;

            // Set session data
            model.text_input.set_session_id(Some(session.id.clone()));
            model.session_state = SessionState::Ready(session);
            model.connection_status = ConnectionStatus::SessionReady;
            model.message_log.touch_scroll();

            // Add the first message to message log
            model
                .message_log
                .create_and_push_user_message(&first_message);

            // Clear pending message
            model.pending_first_message = None;

            // Fetch session messages once session is ready
            if let Some(client) = model.client.clone() {
                (model, Cmd::AsyncLoadSessionMessages(client, session_id))
            } else {
                (model, Cmd::None)
            }
        }

        Msg::SessionCreationFailed(error) => {
            let error_msg = format!("Failed to create session: {}", error);
            model.session_state = SessionState::None;
            model.pending_first_message = None;
            model.transition_to_error(error_msg);
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
            model.message_log.scroll_vertical(&direction);
            (model, Cmd::None)
        }
        Msg::ScrollMessageLogHorizontal(direction) => {
            model.message_log.scroll_horizontal(direction);
            (model, Cmd::None)
        }
        Msg::ValidateScrollPosition(viewport_height, viewport_width) => {
            model
                .message_log
                .validate_scroll_position(viewport_height, viewport_width);
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

        Msg::TerminalResize(_width, _height) => {
            // Terminal resize automatically triggers a re-render
            // No model state changes needed - the view will query current terminal size
            (model, Cmd::None)
        }

        // Session selector messages
        Msg::ShowSessionSelector => {
            model.state = AppState::SelectSession;
            model
                .session_selector
                .handle_event(PopoverSelectorEvent::Show);
            model
                .session_selector
                .handle_event(PopoverSelectorEvent::SetLoading(true));

            // Cache the render height for accurate scroll calculations
            model
                .session_selector
                .cache_render_height_for_terminal(model.init.height());

            // Set current session index if we have an active session
            let current_index = if let Some(current_session) = model.session() {
                // Find the current session in the sessions list
                // Add 1 because "Create New Session" is at index 0
                model
                    .sessions
                    .iter()
                    .position(|s| s.id == current_session.id)
                    .map(|pos| pos + 1)
            } else {
                None
            };

            model
                .session_selector
                .set_current_session_index(current_index);

            if let Some(client) = model.client.clone() {
                (model, Cmd::AsyncLoadSessions(client))
            } else {
                model
                    .session_selector
                    .handle_event(PopoverSelectorEvent::SetError(Some(
                        "No client connection".to_string(),
                    )));
                (model, Cmd::None)
            }
        }

        Msg::SessionSelectorEvent(event) => {
            if let Some(client) = model.client.clone() {
                let changed_index = model.session_selector.handle_event(event.clone());

                if model.change_session(changed_index) {
                    return (model, Cmd::AsyncSpawnSessionInit(client));
                }
            }

            // Handle cancel
            if matches!(event, PopoverSelectorEvent::Cancel) {
                model.state = AppState::Welcome;
            }

            (model, Cmd::None)
        }

        Msg::SessionsLoaded(sessions) => {
            model.sessions = sessions.clone();
            let mut items = vec!["Create New Session".to_string()];
            items.extend(sessions.iter().map(|s| s.title.clone()));
            model
                .session_selector
                .handle_event(PopoverSelectorEvent::SetItems(items));
            //
            // Re-cache the render height since popup size may have changed with new items
            model
                .session_selector
                .cache_render_height_for_terminal(model.init.height());

            // Re-calculate and set current session index after items are loaded
            let current_index = if let Some(current_session) = model.session() {
                model
                    .sessions
                    .iter()
                    .position(|s| s.id == current_session.id)
                    .map(|pos| pos + 1)
            } else {
                None
            };

            model
                .session_selector
                .set_current_session_index(current_index);

            (model, Cmd::None)
        }

        Msg::SessionsLoadFailed(error) => {
            model
                .session_selector
                .handle_event(PopoverSelectorEvent::SetError(Some(format!(
                    "Failed to load sessions: {}",
                    error
                ))));
            (model, Cmd::None)
        }

        Msg::SessionMessagesLoaded(messages) => {
            // Log debug output for fetched messages
            crate::log_debug!("Fetched {} session messages", messages.len());
            model.message_log.set_messages(messages);
            // for (index, message) in messages.iter().enumerate() {
            //     crate::log_debug!("Message {}: {:?}", index + 1, message);
            // }
            (model, Cmd::None)
        }

        Msg::SessionMessagesLoadFailed(error) => {
            crate::log_debug!("Failed to load session messages: {}", error);
            (model, Cmd::None)
        }
    }
}
