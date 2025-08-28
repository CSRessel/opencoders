use crate::{
    app::{
        event_msg::*,
        tea_model::*,
        ui_components::{
            Component, FileSelector, ModalSelectorEvent, MsgModalFileSelector,
            MsgModalSessionSelector, MsgTextArea, SessionSelector, TextInputArea,
        },
    },
    sdk::client::{generate_id, IdPrefix},
};

pub fn update(mut model: &mut Model, msg: Msg) -> CmdOrBatch<Cmd> {
    match msg {
        Msg::ChangeState(new_state) => {
            if matches!(
                new_state,
                AppModalState::Connecting(ConnectionStatus::Connected)
            ) && !model.is_session_ready()
            {
                // Same as selecting the "Create New" option (pending session)
                model.change_session(Some(0));
                return CmdOrBatch::Single(Cmd::None);
            }

            let old_state = model.state.clone();
            model.state = new_state;
            if matches!(old_state, AppModalState::None) {
                model.message_log.touch_scroll();
            }
            CmdOrBatch::Single(Cmd::None)
        }

        // Client initialization messages
        Msg::InitializeClient => {
            model.state = AppModalState::Connecting(ConnectionStatus::Connecting);
            CmdOrBatch::Single(Cmd::AsyncSpawnClientDiscovery)
        }

        Msg::ResponseClientConnect(Ok(client)) => {
            tracing::info!("Client connected successfully");
            model.client = Some(client.clone());
            model.state = AppModalState::Connecting(ConnectionStatus::Connected);
            model.connection_status = ConnectionStatus::Connected;
            // Load modes immediately when client connects
            CmdOrBatch::Single(Cmd::AsyncLoadModes(client))
        }

        Msg::ResponseClientConnect(Err(error)) => {
            let error_msg = format!("Failed to connect to OpenCode server: {}", error);
            model.state = AppModalState::Connecting(ConnectionStatus::Error(error.to_string()));
            CmdOrBatch::Single(Cmd::None)
        }

        // Session management messages
        Msg::ResponseSessionInit(Ok(session)) => {
            let session_id = session.id.clone();
            model.state = AppModalState::None;

            // Set session data
            model.session_state = SessionState::Ready(session);
            model.connection_status = ConnectionStatus::SessionReady;
            model.message_log.touch_scroll();

            // Set session ID in message state
            model.message_state.set_session_id(Some(session_id.clone()));

            // Fetch session messages and start event stream once session is ready
            if let Some(client) = model.client.clone() {
                CmdOrBatch::Batch(vec![
                    Cmd::AsyncLoadSessionMessages(client.clone(), session_id),
                    Cmd::AsyncStartEventStream(client),
                ])
            } else {
                CmdOrBatch::Single(Cmd::None)
            }
        }

        Msg::ResponseSessionCreateWithMessage(Ok((session, first_message))) => {
            let session_id = session.id.clone();
            model.state = AppModalState::None;

            // Set session data
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
                let (provider_id, model_id, mode) = model.get_mode_and_model_settings();
                let message_id = generate_id(IdPrefix::Message);
                model.session_is_idle = false;
                CmdOrBatch::Batch(vec![
                    Cmd::AsyncLoadSessionMessages(client.clone(), session_id.clone()),
                    Cmd::AsyncStartEventStream(client.clone()),
                    Cmd::AsyncSendUserMessage(
                        client.clone(),
                        session_id.clone(),
                        message_id.clone(),
                        first_message.clone(),
                        provider_id,
                        model_id,
                        mode,
                    ),
                ])
            } else {
                CmdOrBatch::Single(Cmd::None)
            }
        }

        Msg::ResponseSessionCreateWithMessage(Err(error)) => {
            let error_msg = format!("Failed to create session: {}", error);
            model.session_state = SessionState::None;
            model.pending_first_message = None;
            model.state = AppModalState::Connecting(ConnectionStatus::Error(error_msg));
            CmdOrBatch::Single(Cmd::None)
        }

        Msg::ResponseSessionInit(Err(error)) => {
            let error_msg = format!("Failed to initialize session: {}", error);
            model.state = AppModalState::Connecting(ConnectionStatus::Error(error_msg));
            CmdOrBatch::Single(Cmd::None)
        }

        Msg::Quit => {
            model.state = AppModalState::Quit;
            CmdOrBatch::Single(Cmd::None)
        }
        Msg::ScrollMessageLog(direction) => {
            model.message_log.scroll_vertical(&direction);
            CmdOrBatch::Single(Cmd::None)
        }
        Msg::ScrollMessageLogHorizontal(direction) => {
            model.message_log.scroll_horizontal(direction);
            CmdOrBatch::Single(Cmd::None)
        }
        Msg::ValidateScrollPosition(viewport_height, viewport_width) => {
            model
                .message_log
                .validate_scroll_position(viewport_height, viewport_width);
            CmdOrBatch::Single(Cmd::None)
        }

        // Task lifecycle messages
        Msg::TaskStarted(_task_id, _description) => {
            // Could update UI to show active tasks
            CmdOrBatch::Single(Cmd::None)
        }

        Msg::TaskCompleted(_task_id) => {
            // Could update UI to remove completed task indicator
            CmdOrBatch::Single(Cmd::None)
        }

        Msg::TaskFailed(_task_id, _error) => {
            // Could show error message or update connection status
            CmdOrBatch::Single(Cmd::None)
        }

        Msg::MarkMessagesViewed => {
            let count = model.messages_needing_stdout_print().len();
            if count > 0 {
                model.mark_messages_printed_to_stdout(count);
            }
            CmdOrBatch::Single(Cmd::None)
        }

        Msg::TerminalResize(_width, _height) => CmdOrBatch::Single(Cmd::TerminalAutoResize),

        Msg::ChangeInlineHeight(new_height) => {
            if model.init.inline_mode() {
                CmdOrBatch::Single(Cmd::TerminalResizeInlineViewport(new_height))
            } else {
                CmdOrBatch::Single(Cmd::None) // No-op if not in inline mode
            }
        }

        Msg::LeaderChangeInline => {
            let new_inline = !model.init.inline_mode().clone();
            model.clear_repeat_leader_timeout();
            CmdOrBatch::Single(Cmd::TerminalRebootWithInline(new_inline))
        }

        // Session selector messages
        Msg::LeaderShowSessionSelector => {
            model.clear_repeat_leader_timeout();
            model.state = AppModalState::ModalSessionSelect;

            // Show the selector using generic event
            let _ = model
                .modal_session_selector
                .modal
                .handle_event(ModalSelectorEvent::Show);

            if let Some(client) = model.client.clone() {
                CmdOrBatch::Batch(vec![
                    Cmd::AsyncLoadSessions(client.clone()),
                    Cmd::AsyncLoadModes(client),
                ])
            } else {
                let _ =
                    model
                        .modal_session_selector
                        .modal
                        .handle_event(ModalSelectorEvent::SetError(Some(
                            "No client connection".to_string(),
                        )));
                CmdOrBatch::Single(Cmd::None)
            }
        }

        Msg::ModalSessionSelector(submsg) => SessionSelector::update(submsg, model),

        Msg::ResponseSessionsLoad(Ok(sessions)) => {
            model.sessions = sessions;

            // Convert sessions to SessionData
            let mut session_data =
                vec![crate::app::ui_components::modal_session_selector::SessionData::new_session()];

            // Re-calculate current session index
            let current_session_id = model.session().map(|s| &s.id);

            for (i, session) in model.sessions.iter().enumerate() {
                let is_current = current_session_id == Some(&session.id);
                session_data.push(
                    crate::app::ui_components::modal_session_selector::SessionData::from_session(
                        session, is_current,
                    ),
                );
            }

            // Set items using the generic event
            let _ = model
                .modal_session_selector
                .modal
                .handle_event(ModalSelectorEvent::SetItems(session_data));

            tracing::debug!("set event for {} sessions!!!", model.sessions.len());
            CmdOrBatch::Single(Cmd::None)
        }

        Msg::ResponseSessionsLoad(Err(error)) => {
            tracing::error!("Failed to load sessions: {}", error);
            let _ = model
                .modal_session_selector
                .modal
                .handle_event(ModalSelectorEvent::SetError(Some(format!(
                    "Failed to load sessions: {}",
                    error
                ))));
            CmdOrBatch::Single(Cmd::None)
        }

        Msg::ResponseModesLoad(Ok(modes)) => {
            model.set_modes(modes);
            CmdOrBatch::Single(Cmd::None)
        }

        Msg::ResponseModesLoad(Err(error)) => {
            tracing::error!("Failed to load modes: {}", error);
            // Don't show error to user for modes loading failure, just log it
            CmdOrBatch::Single(Cmd::None)
        }

        Msg::ResponseSessionMessagesLoad(Ok(messages)) => {
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
            CmdOrBatch::Single(Cmd::None)
        }

        Msg::ResponseSessionMessagesLoad(Err(error)) => {
            tracing::debug!("Failed to load session messages: {}", error);
            CmdOrBatch::Single(Cmd::None)
        }

        Msg::ResponseUserMessageSend(Ok(text)) => {
            tracing::debug!("User message sent successfully: {}", text);
            // Reset idle state since we just sent a message
            model.session_is_idle = false;
            // The message will be received via SSE events and added to message state
            CmdOrBatch::Single(Cmd::None)
        }

        Msg::ResponseUserMessageSend(Err(error)) => {
            tracing::debug!("Failed to send user message: {}", error);
            // Could show error in UI or retry
            CmdOrBatch::Single(Cmd::None)
        }

        Msg::ResponseFileStatusesLoad(Ok(files)) => {
            model.file_status = files.clone();
            // Update the file selector with new data
            model.modal_file_selector.set_files(files);
            CmdOrBatch::Single(Cmd::None)
        }

        Msg::ResponseFileStatusesLoad(Err(error)) => {
            tracing::error!("Failed to load file status: {}", error);
            // Keep the current file status and don't show error to user
            CmdOrBatch::Single(Cmd::None)
        }

        Msg::ResponseFileStatusesLoad(Ok(files)) => {
            model.file_status = files;
            CmdOrBatch::Single(Cmd::None)
        }

        Msg::ResponseFileStatusesLoad(Err(error)) => {
            tracing::error!("Failed to load file status: {}", error);
            // Keep the current file status and don't show error to user
            CmdOrBatch::Single(Cmd::None)
        }

        Msg::CycleModeState => {
            if matches!(model.modes, None) {
                // Request modes from server if empty
                if let Some(client) = model.client.clone() {
                    tracing::debug!("Modes array empty, requesting from server");
                    CmdOrBatch::Single(Cmd::AsyncLoadModes(client))
                } else {
                    tracing::debug!("No client available to load modes");
                    CmdOrBatch::Single(Cmd::None)
                }
            } else {
                model.increment_mode_index();
                CmdOrBatch::Single(Cmd::None)
            }
        }
        Msg::RecordActiveTaskCount(count) => {
            model.active_task_count = count;
            CmdOrBatch::Single(Cmd::None)
        }

        // Event stream messages
        Msg::EventReceived(event) => {
            let cmd = handle_event_received(&mut model, event);
            CmdOrBatch::Single(cmd)
        }

        Msg::EventStreamConnected(event_stream) => {
            tracing::debug!("Event stream connected");
            model.event_stream_state = EventStreamState::Connected(event_stream);
            CmdOrBatch::Single(Cmd::None)
        }

        Msg::EventStreamDisconnected => {
            tracing::debug!("Event stream disconnected");
            model.event_stream_state = EventStreamState::Disconnected;
            CmdOrBatch::Single(Cmd::None)
        }

        Msg::EventStreamError(error) => {
            tracing::debug!("Event stream error: {}", error);
            let cmd = handle_event_stream_error(&mut model, error);
            CmdOrBatch::Single(cmd)
        }

        Msg::EventStreamReconnecting(attempt) => {
            tracing::debug!("Event stream reconnecting (attempt {})", attempt);
            model.event_stream_state = EventStreamState::Reconnecting {
                attempt,
                last_error: "Connection lost".to_string(),
            };
            CmdOrBatch::Single(Cmd::None)
        }

        // Unified repeat shortcut timeout messages
        Msg::RepeatShortcutPressed(key) => {
            model.set_repeat_shortcut_timeout(key);
            CmdOrBatch::Single(Cmd::None)
        }

        Msg::ClearTimeout => {
            model.clear_repeat_shortcut_timeout();
            CmdOrBatch::Single(Cmd::None)
        }

        Msg::SessionAbort => CmdOrBatch::Single(Cmd::AsyncSessionAbort),

        Msg::ToggleVerbosity => {
            model.toggle_verbosity();
            CmdOrBatch::Single(Cmd::None)
        }

        Msg::SubmitTextInput => {
            let text = model.text_input_area.content().trim().to_string();

            // Handle text submission like the legacy SubmitInput logic
            model.input_history.push(text.clone());
            model.last_input = Some(text.clone());

            // If we have a pending session, create it now with this message
            if let SessionState::Pending(pending_info) = &model.session_state {
                if let Some(client) = model.client.clone() {
                    model.session_state = SessionState::Creating(pending_info.clone());
                    model.pending_first_message = Some(text.clone());
                    model.session_is_idle = false;
                    model.text_input_area.clear();
                    return CmdOrBatch::Single(Cmd::AsyncCreateSessionWithMessage(client, text));
                }
            }

            // If we have a ready session, send the message via API
            if let (Some(client), Some(session)) = (model.client.clone(), model.session()) {
                let session_id = session.id.clone();
                let (provider_id, model_id, mode) = model.get_mode_and_model_settings();
                let message_id = generate_id(IdPrefix::Message);
                model.session_is_idle = false;
                model.text_input_area.clear();
                return CmdOrBatch::Single(Cmd::AsyncSendUserMessage(
                    client,
                    session_id,
                    message_id,
                    text,
                    provider_id,
                    model_id,
                    mode,
                ));
            }

            CmdOrBatch::Single(Cmd::None)
        }

        Msg::ModalFileSelector(submsg) => {
            FileSelector::update(submsg, model);
            CmdOrBatch::Single(Cmd::None)
        }

        Msg::TextArea(submsg) => {
            // Special handling for @ symbol when main screen is active
            if let MsgTextArea::KeyInput(key_event) = &submsg {
                if key_event.code == crossterm::event::KeyCode::Char('@')
                    && !key_event
                        .modifiers
                        .contains(crossterm::event::KeyModifiers::SHIFT)
                    && model.is_main_screen_active()
                {
                    // Handle the key input first
                    TextInputArea::update(submsg, &mut model);
                    // Then show file picker and load files
                    model.modal_file_selector.modal.show();
                    model.state = AppModalState::ModalFileSelect;
                    // Load file status if we have a client
                    if let Some(client) = model.client.clone() {
                        return CmdOrBatch::Single(Cmd::AsyncLoadFileStatus(client));
                    } else {
                        return CmdOrBatch::Single(Cmd::None);
                    }
                }
            }

            // Handle component sub-messages using direct method call
            TextInputArea::update(submsg, &mut model);
            CmdOrBatch::Single(Cmd::None)
        }
    }
}

fn handle_event_received(model: &mut Model, event: opencode_sdk::models::Event) -> Cmd {
    use opencode_sdk::models::Event;

    let mut updated = false;

    match event {
        // Message-related events (currently implemented)
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
        Event::MessagePeriodPartPeriodRemoved(_part_remove_event) => {
            // TODO: Handle message part removal
            tracing::debug!("Received message part removed event (not implemented yet)");
        }

        // Session-related events
        Event::SessionPeriodUpdated(session_event) => {
            let updated_session = &*session_event.properties.info;
            tracing::debug!(
                "Received session updated event for session: {}",
                updated_session.id
            );

            // Update sessions list
            if let Some(session_index) = model
                .sessions
                .iter()
                .position(|s| s.id == updated_session.id)
            {
                model.sessions[session_index] = updated_session.clone();
                tracing::debug!("Updated session in sessions list");
            }

            // Update current session if it matches
            if let Some(current_session) = model.session() {
                if current_session.id == updated_session.id {
                    model.session_state = SessionState::Ready(updated_session.clone());
                    tracing::debug!("Updated current session state");
                }
            }
        }
        Event::SessionPeriodDeleted(session_event) => {
            let deleted_session = &*session_event.properties.info;
            tracing::debug!(
                "Received session deleted event for session: {}",
                deleted_session.id
            );

            // Remove from sessions list
            model.sessions.retain(|s| s.id != deleted_session.id);

            // Clear current session if it was the deleted one
            if let Some(current_session) = model.session() {
                if current_session.id == deleted_session.id {
                    tracing::debug!("Deleted session was the current session, clearing state");
                    model.session_state = SessionState::None;
                    model.message_state.clear();
                    model.message_log.set_message_containers(vec![]);

                    model.state = AppModalState::None;
                }
            }
        }
        Event::SessionPeriodIdle(session_event) => {
            let idle_session_id = &session_event.properties.session_id;
            tracing::debug!(
                "Received session idle event for session: {}",
                idle_session_id
            );

            // Update idle state if this is the current session
            if let Some(current_session) = model.session() {
                if current_session.id == *idle_session_id {
                    model.session_is_idle = true;
                    tracing::debug!("Current session is now idle");
                }
            }
        }
        Event::SessionPeriodError(session_event) => {
            let error_props = &session_event.properties;
            tracing::error!(
                "Received session error event: session_id={:?}, error={:?}",
                error_props.session_id,
                error_props.error
            );

            // Show error to user if it's for the current session or no specific session
            let should_show_error = match &error_props.session_id {
                Some(error_session_id) => model
                    .session()
                    .map(|s| &s.id == error_session_id)
                    .unwrap_or(false),
                None => true, // Global error
            };

            if should_show_error {
                let error_msg = if let Some(error) = &error_props.error {
                    format!("Session error: {:?}", error)
                } else {
                    "Unknown session error".to_string()
                };
                model.state = AppModalState::Connecting(ConnectionStatus::Error(error_msg));
            }
        }

        // Permission-related events
        Event::PermissionPeriodUpdated(_permission_event) => {
            // TODO: Handle permission updates
            tracing::debug!("Received permission updated event (not implemented yet)");
        }
        Event::PermissionPeriodReplied(_permission_event) => {
            // TODO: Handle permission replies
            tracing::debug!("Received permission replied event (not implemented yet)");
        }

        // File-related events
        Event::FilePeriodEdited(_file_event) => {
            // TODO: Handle file edits
            tracing::debug!("Received file edited event (not implemented yet)");
        }
        Event::FilePeriodWatcherPeriodUpdated(_file_event) => {
            // TODO: Handle file watcher updates
            tracing::debug!("Received file watcher updated event (not implemented yet)");
        }

        // Storage events
        Event::StoragePeriodWrite(_storage_event) => {
            // TODO: Handle storage writes
            tracing::debug!("Received storage write event (not implemented yet)");
        }

        // System/Infrastructure events
        Event::InstallationPeriodUpdated(_install_event) => {
            // TODO: Handle installation updates
            tracing::debug!("Received installation updated event (not implemented yet)");
        }
        Event::LspPeriodClientPeriodDiagnostics(_lsp_event) => {
            // TODO: Handle LSP diagnostics
            tracing::debug!("Received LSP client diagnostics event (not implemented yet)");
        }
        Event::ServerPeriodConnected(server_event) => {
            tracing::info!("Server health confirmed: {:?}", server_event.properties);

            // Update connection status if currently in error state
            match &model.connection_status {
                ConnectionStatus::Error(_) => {
                    model.connection_status = ConnectionStatus::Connected;
                    tracing::info!("Connection recovered from error state");
                }
                ConnectionStatus::Disconnected => {
                    model.connection_status = ConnectionStatus::Connected;
                    tracing::info!("Server connection established");
                }
                _ => {
                    // Server is healthy, connection status already good
                    tracing::debug!("Server health confirmed, connection already stable");
                }
            }
        }
        Event::IdePeriodInstalled(_ide_event) => {
            // TODO: Handle IDE installation
            tracing::debug!("Received IDE installed event (not implemented yet)");
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
