//! Dual stdout/inline TUI approach:
//!
//! 1. When starting inline, bottom align the TUI:
//!    - If not at bottom of terminal window, move to max row - height - welcome text
//!    - Print welcome text
//!    - scroll down welcome height rows
//!    - Launch inline TUI
//! 2. Render cycle:
//!    - If outside-inline output needed, call view_history for terminal.insert_before
//!    - Render TUI content in fixed viewport at bottom
//! 3. Result: Message content in scrollback history, TUI fixed at terminal bottom
//!
//! The fullscreen viewport has none of these intricacies, because all message
//! history scrolls within the message log.

use crate::{
    app::{
        error::Result,
        event_async_task_manager::AsyncTaskManager,
        event_msg::{Cmd, CmdOrBatch, Msg},
        event_sync_subscriptions,
        tea_model::{AppModalState, ConnectionStatus, Model, ModelInit},
        tea_update::update,
        tea_view::{render_manual_inline_history, view, view_clear},
        terminal::{init_terminal, restore_terminal},
        ui_components::{
            banner::{create_welcome_text, welcome_text_height},
            text_input::TEXT_INPUT_HEIGHT,
        },
    },
    sdk::{extensions::events::EventStream, OpenCodeClient},
};
use crossterm::event;
use eyre::WrapErr;
use ratatui::prelude::Widget;
use ratatui::{backend::CrosstermBackend, crossterm, widgets::Paragraph, Terminal};
use std::io::{self};
use std::time::Duration;
use tokio::time::interval;

pub struct Program {
    model: Model,
    terminal: Option<Terminal<CrosstermBackend<io::Stdout>>>,
    task_manager: AsyncTaskManager,
    needs_render: bool,
}

impl Program {
    pub fn new() -> Result<Self> {
        let model = Model::new();

        let welcome_text = create_welcome_text();
        let mut terminal = init_terminal(&model.init, model.config.height)?;
        terminal.insert_before(welcome_text_height().saturating_add(1), |buf| {
            Paragraph::new(welcome_text).render(buf.area, buf)
        });

        // Create async task manager
        let task_manager = AsyncTaskManager::new();

        Ok(Program {
            model,
            terminal: Some(terminal),
            task_manager,
            needs_render: true, // Initial render needed
        })
    }

    pub fn run(self) -> Result<()> {
        // Create a Tokio runtime for this blocking function
        let runtime = tokio::runtime::Runtime::new()?;
        runtime.block_on(self.run_async())
    }

    async fn run_async(mut self) -> Result<()> {
        // Create tick interval for periodic updates (60 FPS) - must be inside tokio runtime
        let mut tick_interval = interval(Duration::from_millis(4));

        // Auto-trigger client discovery at startup
        self.spawn_command(Cmd::AsyncSpawnClientDiscovery).await?;

        loop {
            // Check for quit state
            if matches!(self.model.state, AppModalState::Quit) {
                break;
            }

            // Process all available events and messages first
            let mut had_events = false;

            // Check for async task completions (non-blocking)
            let async_messages = self.task_manager.poll_messages();
            if !async_messages.is_empty() {
                had_events = true;
                for msg in async_messages {
                    let cmd = update(&mut self.model, msg);
                    self.needs_render = true;
                    self.spawn_commands(cmd).await?;
                }
            }

            // Check for input events (non-blocking)
            if let Some(msg) = self.poll_input_events().await? {
                had_events = true;
                let cmd = update(&mut self.model, msg);
                self.needs_render = true;
                self.spawn_commands(cmd).await?;
            }

            // Check for SSE events (non-blocking)
            if self.poll_sse_events().await? {
                had_events = true;
            }

            // If we had events, continue loop immediately to process more
            if had_events {
                continue;
            }

            // No events - wait for either a tick or go back to polling
            tokio::select! {
                // Periodic tick for cleanup and rendering
                _ = tick_interval.tick() => {
                    // Cleanup completed tasks periodically
                    self.task_manager.cleanup_completed_tasks();

                    // Check for expired timeouts and process them
                    let expired_timeouts = self.model.get_expired_timeouts();
                    for timeout_type in expired_timeouts {
                        let cmd = update(&mut self.model, Msg::TimeoutExpired(timeout_type));
                        self.needs_render = true;
                        self.spawn_commands(cmd).await?;
                    }

                    // Only render if needed
                    if self.needs_render {
                        self.render_view().await?;

                        self.needs_render = false;
                    }
                },
            }
        }
        Ok(())
    }

    async fn render_view(&mut self) -> Result<()> {
        let cmd = update(
            &mut self.model,
            Msg::RecordActiveTaskCount(self.task_manager.active_task_count()),
        );
        self.spawn_commands(cmd).await?;

        // View: Manual rendering outside the TUI viewport
        if self.model.needs_manual_output() {
            if let Some(terminal) = self.terminal.as_mut() {
                // // Clear the TUI
                // terminal.draw(|f| view_clear(f))?;

                // Manually execute with crossterm
                render_manual_inline_history(&self.model, terminal)?;
            }
        }

        // View: Pure rendering, within the TUI
        if let Some(terminal) = self.terminal.as_mut() {
            terminal.draw(|f| view(&self.model, f))?;
        }
        let cmd = update(&mut self.model, Msg::MarkMessagesViewed);
        self.spawn_commands(cmd).await?;

        Ok(())
    }

    async fn poll_input_events(&self) -> Result<Option<Msg>> {
        // Check if we should listen for input events
        let subs = crate::app::event_sync_subscriptions::subscriptions(&self.model);

        if !subs.contains(&crate::app::event_msg::Sub::KeyboardInput) {
            return Ok(None);
        }

        // Use async crossterm event polling
        if event::poll(Duration::from_millis(0))? {
            let event = event::read()?;
            return Ok(crate::app::event_sync_subscriptions::crossterm_to_msg(
                event,
                &self.model,
            ));
        }

        Ok(None)
    }

    async fn poll_sse_events(&mut self) -> Result<bool> {
        use crate::app::event_msg::Sub;
        use crate::app::tea_model::EventStreamState;

        // Only poll if the model is subscribed to the event stream
        if !event_sync_subscriptions::subscriptions(&self.model).contains(&Sub::EventStream) {
            return Ok(false);
        }

        let mut events = Vec::new();
        if let EventStreamState::Connected(event_stream) = &mut self.model.event_stream_state {
            // Loop to drain all pending events from the stream's buffer
            while let Some(event) = event_stream.try_next_event() {
                events.push(event);
            }
        }

        if !events.is_empty() {
            let mut processed_event = false;
            for event in events {
                let cmd = update(&mut self.model, Msg::EventReceived(event));
                self.needs_render = true; // Signal that a re-render is needed
                self.spawn_commands(cmd).await?;
                processed_event = true;
            }
            Ok(processed_event)
        } else {
            Ok(false)
        }
    }

    async fn spawn_commands(&mut self, cmds: CmdOrBatch<Cmd>) -> Result<()> {
        match cmds {
            CmdOrBatch::Single(cmd) => {
                self.spawn_command(cmd).await?;
            }
            CmdOrBatch::Batch(commands) => {
                // Handle batch commands by processing them in the main loop
                // rather than recursively to avoid infinite future size
                for cmd in commands {
                    match cmd {
                        Cmd::AsyncSpawnClientDiscovery
                        | Cmd::AsyncSpawnSessionInit(_)
                        | Cmd::AsyncCreateSessionWithMessage(_, _)
                        | Cmd::AsyncLoadSessions(_)
                        | Cmd::AsyncLoadModes(_)
                        | Cmd::AsyncLoadSessionMessages(_, _)
                        | Cmd::AsyncLoadFileStatus(_)
                        | Cmd::AsyncLoadFindFiles(_, _)
                        | Cmd::AsyncSendUserMessage(_, _, _, _, _, _, _)
                        | Cmd::AsyncSendUserMessageWithAttachments(_, _, _, _, _, _, _, _)
                        | Cmd::AsyncCancelTask(_)
                        | Cmd::AsyncSessionAbort
                        | Cmd::AsyncStartEventStream(_)
                        | Cmd::AsyncStopEventStream
                        | Cmd::AsyncReconnectEventStream
                        | Cmd::TerminalRebootWithInline(_)
                        | Cmd::TerminalResizeInlineViewport(_)
                        | Cmd::TerminalScrollPastHeight
                        | Cmd::TerminalAutoResize => {
                            Box::pin(self.spawn_command(cmd)).await?;
                        }
                        Cmd::None => {}
                    }
                }
            }
        };
        Ok(())
    }

    async fn spawn_command(&mut self, cmd: Cmd) -> Result<()> {
        match cmd {
            Cmd::TerminalRebootWithInline(new_inline_mode) => {
                // Deconstruct the old terminal by taking ownership from the Option
                let mut old_terminal = self.terminal.take();

                if !new_inline_mode {
                    if let Some(terminal) = old_terminal.as_mut() {
                        // Clear the TUI when leaving inline mode, so it doesn't
                        // leave artifacts in the history
                        tracing::debug!("Clearing to switch from inline to altscreen");
                        terminal.draw(|f| view_clear(f))?;
                        // Move the cursor back to the top left of the TUI,
                        // so if we switch back and forth we don't offset
                        terminal
                            .draw(|f| f.set_cursor_position((f.area().left(), f.area().top())))?;
                    }
                }

                // Restore the old terminal state before creating new one
                if let Some(mut terminal) = old_terminal.take() {
                    restore_terminal(&self.model.init, self.model.config.height)
                        .wrap_err("Failed to restore terminal")?;
                }
                let new_init = ModelInit::new(new_inline_mode);
                let mut terminal = init_terminal(&new_init, self.model.config.height)?;
                self.terminal = Some(terminal);
                self.model.init = new_init;
            }

            Cmd::TerminalResizeInlineViewport(new_height) => {
                if let Some(terminal) = self.terminal.as_mut() {
                    if self.model.init.inline_mode() {
                        // Update model state first
                        self.model.config.height = new_height;

                        // Use ratatui's resize method with new inline viewport
                        let terminal_size = terminal.size()?;
                        let new_viewport_area =
                            ratatui::layout::Rect::new(0, 0, terminal_size.width, new_height);
                        terminal.resize(new_viewport_area)?;

                        // Force re-render
                        self.needs_render = true;
                    }
                }
            }

            Cmd::TerminalAutoResize => {
                if let Some(terminal) = self.terminal.as_mut() {
                    terminal.autoresize()?;
                    self.needs_render = true;
                }
            }

            Cmd::TerminalScrollPastHeight => {
                // Inline mode text input will have some stdout messages in
                // viewport, so switching screens we have to push that up

                if let Some(terminal) = self.terminal.as_mut() {
                    // Clear the TUI
                    terminal.draw(|f| view_clear(f))?;

                    // Rows of message output that need to be moved up,
                    // before more TUI can be rendered
                    let scroll_line_count = self.model.config.height - TEXT_INPUT_HEIGHT;
                    crossterm::execute!(
                        io::stdout(),
                        crossterm::terminal::ScrollUp(scroll_line_count)
                    )?;
                    self.needs_render = true;
                }
            }

            Cmd::AsyncSpawnClientDiscovery => {
                // Spawn async client discovery task
                self.task_manager.spawn_task(async move {
                    match OpenCodeClient::discover().await {
                        Ok(client) => Msg::ResponseClientConnect(Ok(client)),
                        Err(error) => Msg::ResponseClientConnect(Err(error)),
                    }
                });
            }

            Cmd::AsyncSpawnSessionInit(client) => {
                // Check if there's a selected session from the session selector
                let selected_session_id = self.model.current_session_id();

                // Spawn async session initialization task
                self.task_manager.spawn_task(async move {
                    // If we have a selected session ID, save it as the last session first
                    if let Some(session_id) = selected_session_id {
                        if let Err(e) = client.switch_to_session(&session_id).await {
                            tracing::error!("Save session ID {} failed: {}", session_id, e);
                        }
                    } else {
                        if let Err(e) = client.clear_current_session().await {
                            tracing::error!("Clear session failed: {}", e);
                        }
                    }

                    // Get or create session (will use saved session if available)
                    match client.get_or_create_session().await {
                        Ok(session) => Msg::ResponseSessionInit(Ok(session)),
                        Err(error) => Msg::ResponseSessionInit(Err(error)),
                    }
                });
            }

            Cmd::AsyncCreateSessionWithMessage(client, first_message) => {
                // Spawn async session creation task with first message
                self.task_manager.spawn_task(async move {
                    // Clear any existing session first
                    if let Err(error) = client.clear_current_session().await {
                        tracing::error!("Clear session failed: {}", error);
                        Msg::ResponseSessionCreateWithMessage(Err(error))
                    } else {
                        // Create new session
                        match client.create_new_session().await {
                            Ok(session) => {
                                Msg::ResponseSessionCreateWithMessage(Ok((session, first_message)))
                            }
                            Err(error) => {
                                tracing::error!("Create session failed: {}", error);
                                Msg::ResponseSessionCreateWithMessage(Err(error))
                            }
                        }
                    }
                });
            }

            Cmd::AsyncLoadSessions(client) => {
                // Spawn async session loading task
                self.task_manager.spawn_task(async move {
                    match client.list_sessions().await {
                        Ok(sessions) => Msg::ResponseSessionsLoad(Ok(sessions)),
                        Err(error) => Msg::ResponseSessionsLoad(Err(error)),
                    }
                });
            }

            Cmd::AsyncLoadFileStatus(client) => {
                // Spawn async file status loading task
                self.task_manager.spawn_task(async move {
                    match client.get_file_status().await {
                        Ok(file_status) => Msg::ResponseFileStatusesLoad(Ok(file_status)),
                        Err(error) => Msg::ResponseFileStatusesLoad(Err(error)),
                    }
                });
            }

            Cmd::AsyncLoadFindFiles(client, query) => {
                // Spawn async find files task
                self.task_manager.spawn_task(async move {
                    match client.find_files(&query).await {
                        Ok(file_paths) => Msg::ResponseFindFiles(Ok(file_paths)),
                        Err(error) => Msg::ResponseFindFiles(Err(error)),
                    }
                });
            }

            Cmd::AsyncLoadModes(client) => {
                // Spawn async modes loading task
                self.task_manager.spawn_task(async move {
                    match client.get_agent_configs().await {
                        Ok(agent_configs) => Msg::ResponseModesLoad(Ok(agent_configs)),
                        Err(error) => Msg::ResponseModesLoad(Err(error)),
                    }
                });
            }

            Cmd::AsyncLoadSessionMessages(client, session_id) => {
                // Spawn async session messages loading task
                self.task_manager.spawn_task(async move {
                    match client.get_messages(&session_id).await {
                        Ok(messages) => Msg::ResponseSessionMessagesLoad(Ok(messages)),
                        Err(error) => Msg::ResponseSessionMessagesLoad(Err(error)),
                    }
                });
            }

            Cmd::AsyncSendUserMessage(
                client,
                session_id,
                message_id,
                text,
                provider_id,
                model_id,
                mode,
            ) => {
                // Spawn async user message sending task
                self.task_manager.spawn_task(async move {
                    // Convert Mode object to string for API call
                    match client
                        .send_user_message(
                            &session_id,
                            &message_id,
                            &text,
                            &provider_id,
                            &model_id,
                            mode.as_deref(),
                        )
                        .await
                    {
                        Ok(_) => Msg::ResponseUserMessageSend(Ok(text)),
                        Err(error) => Msg::ResponseUserMessageSend(Err(error)),
                    }
                });
            }

            Cmd::AsyncSendUserMessageWithAttachments(
                client,
                session_id,
                message_id,
                text,
                attached_files,
                provider_id,
                model_id,
                mode,
            ) => {
                // Spawn async user message with attachments sending task
                self.task_manager.spawn_task(async move {
                    match client
                        .send_user_message_with_attachments(
                            &session_id,
                            &message_id,
                            &text,
                            &attached_files,
                            &provider_id,
                            &model_id,
                            mode.as_deref(),
                        )
                        .await
                    {
                        Ok(_) => Msg::ResponseUserMessageSend(Ok(text)),
                        Err(error) => Msg::ResponseUserMessageSend(Err(error)),
                    }
                });
            }

            Cmd::AsyncSessionAbort => {
                self.task_manager.spawn_task(async move {
                    Msg::ChangeState(AppModalState::Connecting(ConnectionStatus::Connected))
                    // Will reset other necessary state to delect session
                });
            }

            Cmd::AsyncCancelTask(task_id) => {
                self.task_manager.cancel_task(task_id);
            }

            Cmd::AsyncStartEventStream(client) => {
                // Spawn async event stream initialization task
                self.task_manager.spawn_task(async move {
                    match EventStream::new(client.configuration().clone()).await {
                        Ok(event_stream) => {
                            let handle = event_stream.handle();
                            Msg::EventStreamConnected(handle)
                        }
                        Err(error) => Msg::EventStreamError(format!(
                            "Failed to start event stream: {}",
                            error
                        )),
                    }
                });
            }

            Cmd::AsyncStopEventStream => {
                // Event stream will be dropped when the handle is removed from the model
                // No explicit action needed as the EventStream handles cleanup internally
            }

            Cmd::AsyncReconnectEventStream => {
                // For now, we'll just try to reconnect after a delay
                // In a real implementation, you might want to use the existing client
                self.task_manager.spawn_task(async move {
                    tokio::time::sleep(Duration::from_millis(1000)).await;
                    Msg::EventStreamError("Reconnection not implemented yet".to_string())
                });
            }

            Cmd::None => {}
        }
        Ok(())
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        if let Some(_) = self.terminal.take() {
            if let Err(e) = restore_terminal(&self.model.init, self.model.config.height) {
                tracing::error!("Failed to restore terminal during program cleanup: {}", e);
                eprintln!(
                    "Failed to restore terminal. Run `reset` or restart your terminal to recover: {}",
                    e
                );
            }
        }
    }
}
