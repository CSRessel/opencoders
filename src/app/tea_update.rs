use crate::app::{
    event_msg::*,
    tea_model::*,
    tracing_macros::{debug_hot_path, trace_hot_path},
    ui_components::{text_input::TextInputEvent, PopoverSelectorEvent},
};
use opencode_sdk::models::{
    Message, Part, SessionByIdMessage200ResponseInner, TextPart, UserMessage,
};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static COUNTER: AtomicU32 = AtomicU32::new(0);

fn generate_message_id() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;

    let counter = COUNTER.fetch_add(1, Ordering::SeqCst);
    let timestamp_with_counter = (now << 12) + (counter as u64 & 0xFFF);

    // Convert to hex manually
    let time_bytes = timestamp_with_counter.to_be_bytes();
    let time_hex = time_bytes[2..8]
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>();

    // Generate random base62 string using system entropy
    let random_part = (0..14)
        .map(|i| {
            let entropy = (now
                .wrapping_add(i as u64)
                .wrapping_mul(1103515245)
                .wrapping_add(12345))
                >> 16;
            let chars = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
            chars.chars().nth((entropy % 62) as usize).unwrap()
        })
        .collect::<String>();

    format!("msg_{}{}", time_hex, random_part)
}

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

                // If we have a ready session, send the message via API
                if let (Some(client), Some(session)) = (model.client.clone(), model.session()) {
                    let session_id = session.id.clone();
                    let (provider_id, model_id, mode) =
                        if let Some(current_mode) = model.get_current_mode() {
                            // Use mode's model info if available, otherwise fall back to SDK defaults
                            let provider = current_mode
                                .model
                                .as_ref()
                                .map(|m| m.provider_id.clone())
                                .unwrap_or_else(|| model.sdk_provider.clone());
                            let model_name = current_mode
                                .model
                                .as_ref()
                                .map(|m| m.model_id.clone())
                                .unwrap_or_else(|| model.sdk_model.clone());
                            (provider, model_name, Some(current_mode.clone()))
                        } else {
                            // Fallback to hardcoded values if no mode selected
                            tracing::debug!("No mode selected, using fallback provider/model");
                            (model.sdk_provider.clone(), model.sdk_model.clone(), None)
                        };
                    let message_id = generate_message_id();
                    return (
                        model,
                        Cmd::AsyncSendUserMessage(
                            client,
                            session_id,
                            message_id,
                            submitted_text,
                            provider_id,
                            model_id,
                            mode,
                        ),
                    );
                };
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

            let old_state = model.state;
            model.state = new_state.clone();
            if matches!(old_state, AppState::TextEntry) {
                // TODO we need to crossterm scroll down height many lines
                // when coming from inline mode first...
                model.clear_input_state();
                if model.init.inline_mode() {
                    (model, Cmd::TerminalScrollPastHeight)
                } else {
                    (model, Cmd::None)
                }
            } else {
                if matches!(model.state, AppState::TextEntry) {
                    // Auto-scroll to bottom when entering text entry mode
                    model.message_log.touch_scroll();
                }
                (model, Cmd::None)
            }
        }

        // Client initialization messages
        Msg::InitializeClient => {
            model.transition_to_connecting();
            (model, Cmd::AsyncSpawnClientDiscovery)
        }

        Msg::ClientConnected(client) => {
            tracing::info!("Client connected successfully");
            model.client = Some(client.clone());
            model.transition_to_connected();
            // Load modes immediately when client connects
            (model, Cmd::AsyncLoadModes(client))
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

            // Set session ID in message state
            model.message_state.set_session_id(Some(session_id.clone()));

            // Fetch session messages and start event stream once session is ready
            if let Some(client) = model.client.clone() {
                (
                    model,
                    Cmd::Batch(vec![
                        Cmd::AsyncLoadSessionMessages(client.clone(), session_id),
                        Cmd::AsyncStartEventStream(client),
                    ]),
                )
            } else {
                (model, Cmd::None)
            }
        }

        Msg::SessionCreatedWithMessage(session, first_message) => {
            let session_id = session.id.clone();
            model.state = AppState::TextEntry;

            // Set session data
            model.text_input.set_session_id(Some(session.id.clone()));
            model.session_state = SessionState::Ready(session.clone());
            model.connection_status = ConnectionStatus::SessionReady;
            model.message_log.touch_scroll();

            // Set session ID in message state
            model.message_state.set_session_id(Some(session_id.clone()));

            // Clear pending message
            model.pending_first_message = None;

            // Fetch session messages and start event stream once session is ready
            if let Some(client) = model.client.clone() {
                let session_id = session.id.clone();
                let (provider_id, model_id, mode) =
                    if let Some(current_mode) = model.get_current_mode() {
                        // Use mode's model info if available, otherwise fall back to SDK defaults
                        let provider = current_mode
                            .model
                            .as_ref()
                            .map(|m| m.provider_id.clone())
                            .unwrap_or_else(|| model.sdk_provider.clone());
                        let model_name = current_mode
                            .model
                            .as_ref()
                            .map(|m| m.model_id.clone())
                            .unwrap_or_else(|| model.sdk_model.clone());
                        (provider, model_name, Some(current_mode.clone()))
                    } else {
                        // Fallback to hardcoded values if no mode selected
                        tracing::debug!(
                            "No mode selected for session creation, using fallback provider/model"
                        );
                        (model.sdk_provider.clone(), model.sdk_model.clone(), None)
                    };
                let message_id = generate_message_id();
                (
                    model,
                    Cmd::Batch(vec![
                        Cmd::AsyncLoadSessionMessages(client.clone(), session_id.clone()),
                        Cmd::AsyncStartEventStream(client.clone()),
                        Cmd::AsyncSendUserMessage(
                            client.clone(),
                            session_id.clone(),
                            message_id.clone(),
                            first_message.clone(),
                            provider_id.clone(),
                            model_id.clone(),
                            mode.clone(),
                        ),
                    ]),
                )
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
            (model, Cmd::TerminalRebootWithInline(new_inline))
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
            if count > 0 {
                model.mark_messages_printed_to_stdout(count);
            }
            (model, Cmd::None)
        }

        Msg::TerminalResize(_width, _height) => {
            // Enhanced to trigger autoresize for seamless viewport updates
            if model.state == AppState::TextEntry {
                (
                    model,
                    Cmd::Batch(vec![Cmd::TerminalScrollPastHeight, Cmd::TerminalAutoResize]),
                )
            } else {
                (model, Cmd::TerminalAutoResize)
            }
        }

        Msg::ChangeInlineHeight(new_height) => {
            if model.init.inline_mode() {
                if model.state == AppState::TextEntry {
                    (
                        model,
                        Cmd::Batch(vec![
                            Cmd::TerminalScrollPastHeight,
                            Cmd::TerminalResizeInlineViewport(new_height),
                        ]),
                    )
                } else {
                    (model, Cmd::TerminalResizeInlineViewport(new_height))
                }
            } else {
                (model, Cmd::None) // No-op if not in inline mode
            }
        }

        // Session selector messages
        Msg::ShowSessionSelector => {
            let prefix_cmd = match model.state {
                AppState::TextEntry => Cmd::TerminalScrollPastHeight,
                _ => Cmd::None,
            };

            model.state = AppState::SelectSession;
            model
                .session_selector
                .cache_render_height_for_terminal(model.config.height);

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

            // Make the selector visible
            model
                .session_selector
                .handle_event(PopoverSelectorEvent::Show);

            if let Some(client) = model.client.clone() {
                tracing::debug!("waiting for session load......");
                (
                    model,
                    Cmd::Batch(vec![
                        prefix_cmd,
                        Cmd::AsyncLoadSessions(client.clone()),
                        Cmd::AsyncLoadModes(client),
                    ]),
                )
            } else {
                tracing::debug!("no client yet......");
                model
                    .session_selector
                    .handle_event(PopoverSelectorEvent::SetError(Some(
                        "No client connection".to_string(),
                    )));
                (model, prefix_cmd)
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
                .cache_render_height_for_terminal(model.config.height);

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
            tracing::error!("Failed to load sessions: {}", error);
            model
                .session_selector
                .handle_event(PopoverSelectorEvent::SetError(Some(format!(
                    "Failed to load sessions: {}",
                    error
                ))));
            (model, Cmd::None)
        }

        Msg::ModesLoaded(modes) => {
            model.set_modes(modes);
            (model, Cmd::None)
        }

        Msg::ModesLoadFailed(error) => {
            tracing::error!("Failed to load modes: {}", error);
            // Don't show error to user for modes loading failure, just log it
            (model, Cmd::None)
        }

        Msg::CycleModeState => {
            if model.modes.is_empty() {
                // Request modes from server if empty
                if let Some(client) = model.client.clone() {
                    tracing::debug!("Modes array empty, requesting from server");
                    (model, Cmd::AsyncLoadModes(client))
                } else {
                    tracing::debug!("No client available to load modes");
                    (model, Cmd::None)
                }
            } else {
                // Cycle through modes
                let next_index = match model.mode_state {
                    None => {
                        tracing::debug!("No mode selected, setting to first mode (index 0)");
                        Some(0)
                    }
                    Some(current) => {
                        if current >= model.modes.len() {
                            tracing::debug!(
                                "Current mode index {} out of bounds, resetting to 0",
                                current
                            );
                            Some(0)
                        } else {
                            let next = (current + 1) % model.modes.len();
                            tracing::debug!("Cycling from mode {} to mode {}", current, next);
                            Some(next)
                        }
                    }
                };
                model.set_mode_by_index(next_index);
                (model, Cmd::None)
            }
        }

        Msg::SessionMessagesLoaded(messages) => {
            // Log debug output for fetched messages
            tracing::debug!("Fetched {} session messages", messages.len());
            model.message_state.load_messages(messages.clone());
            let message_containers = model
                .message_state
                .get_all_message_containers()
                .into_iter()
                .cloned()
                .collect();
            model.message_log.set_message_containers(message_containers);
            (model, Cmd::None)
        }

        Msg::SessionMessagesLoadFailed(error) => {
            tracing::debug!("Failed to load session messages: {}", error);
            (model, Cmd::None)
        }

        Msg::UserMessageSent(text) => {
            tracing::debug!("User message sent successfully: {}", text);
            // The message will be received via SSE events and added to message state
            (model, Cmd::None)
        }

        Msg::UserMessageSendFailed(error) => {
            tracing::debug!("Failed to send user message: {}", error);
            // Could show error in UI or retry
            (model, Cmd::None)
        }

        // Event stream messages
        Msg::EventReceived(event) => {
            let cmd = handle_event_received(&mut model, event);
            (model, cmd)
        }

        Msg::EventStreamConnected(event_stream) => {
            tracing::debug!("Event stream connected");
            model.event_stream_state = EventStreamState::Connected(event_stream);
            (model, Cmd::None)
        }

        Msg::EventStreamDisconnected => {
            tracing::debug!("Event stream disconnected");
            model.event_stream_state = EventStreamState::Disconnected;
            (model, Cmd::None)
        }

        Msg::EventStreamError(error) => {
            tracing::debug!("Event stream error: {}", error);
            let cmd = handle_event_stream_error(&mut model, error);
            (model, cmd)
        }

        Msg::EventStreamReconnecting(attempt) => {
            tracing::debug!("Event stream reconnecting (attempt {})", attempt);
            model.event_stream_state = EventStreamState::Reconnecting {
                attempt,
                last_error: "Connection lost".to_string(),
            };
            (model, Cmd::None)
        }

        // Unified repeat shortcut timeout messages
        Msg::RepeatShortcutPressed(key) => {
            model.set_repeat_shortcut_timeout(key);
            (model, Cmd::None)
        }

        Msg::ClearTimeout => {
            model.clear_repeat_shortcut_timeout();
            (model, Cmd::None)
        }

        Msg::SessionAbort => (model, Cmd::AsyncSessionAbort),
    }
}

fn handle_event_received(model: &mut Model, event: opencode_sdk::models::Event) -> Cmd {
    use opencode_sdk::models::Event;

    let mut updated = false;

    match event {
        Event::MessagePeriodUpdated(msg_event) => {
            if model
                .message_state
                .update_message(*msg_event.properties.info)
            {
                updated = true;
                tracing::debug!("Updated message from event");
            }
        }
        Event::MessagePeriodPartPeriodUpdated(part_event) => {
            if model
                .message_state
                .update_message_part(*part_event.properties.part)
            {
                updated = true;
                tracing::debug!("Updated message part from event");
            }
        }
        Event::MessagePeriodRemoved(remove_event) => {
            if model.message_state.remove_message(
                &remove_event.properties.session_id,
                &remove_event.properties.message_id,
            ) {
                updated = true;
                tracing::debug!("Removed message from event");
            }
        }
        _ => {
            // Ignore non-message events for now
        }
    }

    if updated {
        // Update the message log with the new state
        let message_containers = model
            .message_state
            .get_all_message_containers()
            .into_iter()
            .cloned()
            .collect();
        model.message_log.set_message_containers(message_containers);
    }

    Cmd::None
}

fn handle_event_stream_error(model: &mut Model, error: String) -> Cmd {
    match &model.event_stream_state {
        EventStreamState::Connected(_) => {
            // First failure - attempt reconnection
            model.event_stream_state = EventStreamState::Reconnecting {
                attempt: 1,
                last_error: error.clone(),
            };
            Cmd::AsyncReconnectEventStream
        }
        EventStreamState::Reconnecting { attempt, .. } if *attempt < 3 => {
            // Retry up to 3 times
            model.event_stream_state = EventStreamState::Reconnecting {
                attempt: attempt + 1,
                last_error: error.clone(),
            };
            Cmd::AsyncReconnectEventStream
        }
        _ => {
            // Give up after 3 attempts
            model.event_stream_state = EventStreamState::Failed(error);
            Cmd::None
        }
    }
}
