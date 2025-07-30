use crate::{
    app::{
        event_async_task_manager::AsyncTaskManager,
        event_msg::{Cmd, Msg},
        event_sync_subscriptions::poll_subscriptions,
        tea_model::{AppState, Model, ModelInit},
        tea_update::update,
        tea_view::{view, view_clear, view_manual},
        terminal::TerminalGuard,
        ui_components::banner::create_welcome_text,
    },
    log_error,
    sdk::OpenCodeClient,
};
use crossterm::event::{self, Event};
use ratatui::{backend::CrosstermBackend, style::Color, text::Text, Terminal};
use std::io::{self, Write};
use std::time::Duration;
use tokio::time::{interval, Interval};

pub struct Program {
    model: Model,
    terminal: Option<Terminal<CrosstermBackend<io::Stdout>>>,
    guard: Option<TerminalGuard>,
    task_manager: AsyncTaskManager,
    needs_render: bool,
}

impl Program {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let model = Model::new();

        // Print welcome message to stdout before entering TUI
        let welcome_text = create_welcome_text();
        print!("{}\n\n\n", welcome_text);

        let (guard, terminal) = TerminalGuard::new(&model.init, model.config.height)?;

        // Create async task manager
        let task_manager = AsyncTaskManager::new();

        Ok(Program {
            model,
            terminal: Some(terminal),
            guard: Some(guard),
            task_manager,
            needs_render: true, // Initial render needed
        })
    }

    pub fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        // Create a Tokio runtime for this blocking function
        let runtime = tokio::runtime::Runtime::new()?;
        runtime.block_on(self.run_async())
    }

    async fn run_async(mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Create tick interval for periodic updates (60 FPS) - must be inside tokio runtime
        let mut tick_interval = interval(Duration::from_millis(16));

        // Auto-trigger client discovery at startup
        self.spawn_command(Cmd::AsyncSpawnClientDiscovery).await?;

        loop {
            // Check for quit state
            if matches!(self.model.state, AppState::Quit) {
                break;
            }

            // Process all available events and messages first
            let mut had_events = false;

            // Check for async task completions (non-blocking)
            let async_messages = self.task_manager.poll_messages();
            if !async_messages.is_empty() {
                had_events = true;
                for msg in async_messages {
                    let (new_model, cmd) = update(self.model, msg);
                    self.model = new_model;
                    self.needs_render = true;
                    self.spawn_command(cmd).await?;
                }
            }

            // Check for input events (non-blocking)
            if let Some(msg) = self.poll_input_events().await? {
                had_events = true;
                let (new_model, cmd) = update(self.model, msg);
                self.model = new_model;
                self.needs_render = true;
                self.spawn_command(cmd).await?;
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

                    // Only render if needed
                    if self.needs_render {
                        self.render_view()?;
                        let (new_model, cmd) = update(self.model, Msg::MarkMessagesViewed);
                        self.model = new_model;
                        self.spawn_command(cmd).await?;
                        self.needs_render = false;
                    }
                },
            }
        }
        Ok(())
    }

    fn render_view(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // View: Manual rendering outside the TUI viewport
        if self.model.needs_manual_output() {
            if let Some(terminal) = self.terminal.as_mut() {
                // Clear the TUI
                terminal.draw(|f| view_clear(f))?;

                // Manually execute with crossterm
                view_manual(&self.model)?;
            }
        }

        // View: Pure rendering, within the TUI
        if let Some(terminal) = self.terminal.as_mut() {
            terminal.draw(|f| view(&self.model, f))?;
        }

        Ok(())
    }

    async fn poll_input_events(&self) -> Result<Option<Msg>, Box<dyn std::error::Error>> {
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

    async fn spawn_command(&mut self, cmd: Cmd) -> Result<(), Box<dyn std::error::Error>> {
        match cmd {
            Cmd::RebootTerminalWithInline(inline_mode) => {
                // Deconstruct the old terminal by taking ownership from the Option
                let old_guard = self.guard.take();
                let mut old_terminal = self.terminal.take();

                if !inline_mode {
                    if let Some(terminal) = old_terminal.as_mut() {
                        // Clear the TUI when leaving inline mode, so it doesn't
                        // leave artifacts in the history
                        terminal.draw(|f| view_clear(f))?;
                        // Move the cursor back to the top left of the TUI,
                        // so if we switch back and forth we don't offset
                        terminal
                            .draw(|f| f.set_cursor_position((f.area().left(), f.area().top())))?;
                    }
                };

                // Explicitly drop the old guard and terminal
                drop(old_guard);
                drop(old_terminal);

                let new_init = ModelInit::new(inline_mode);
                let (guard, terminal) = TerminalGuard::new(&new_init, self.model.config.height)?;
                self.guard = Some(guard);
                self.terminal = Some(terminal);
                self.model.init = new_init;
            }

            Cmd::ResizeInlineViewport(new_height) => {
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

            Cmd::AutoResizeTerminal => {
                if let Some(terminal) = self.terminal.as_mut() {
                    terminal.autoresize()?;
                    self.needs_render = true;
                }
            }

            Cmd::AsyncSpawnClientDiscovery => {
                // Spawn async client discovery task
                self.task_manager.spawn_task(async move {
                    match OpenCodeClient::discover().await {
                        Ok(client) => Msg::ClientConnected(client),
                        Err(error) => Msg::ClientConnectionFailed(error),
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
                            log_error!("save session ID {} failed: {}", session_id, e);
                        }
                    } else {
                        if let Err(e) = client.clear_current_session().await {
                            log_error!("clear session failed: {}", e);
                        }
                    }

                    // Get or create session (will use saved session if available)
                    match client.get_or_create_session().await {
                        Ok(session) => Msg::SessionReady(session),
                        Err(error) => Msg::SessionInitializationFailed(error),
                    }
                });
            }

            Cmd::AsyncCreateSessionWithMessage(client, first_message) => {
                // Spawn async session creation task with first message
                self.task_manager.spawn_task(async move {
                    // Clear any existing session first
                    if let Err(error) = client.clear_current_session().await {
                        log_error!("clear session failed: {}", error);
                        Msg::SessionCreationFailed(error)
                    } else {
                        // Create new session
                        match client.create_new_session().await {
                            Ok(session) => Msg::SessionCreatedWithMessage(session, first_message),
                            Err(error) => {
                                log_error!("create session failed: {}", error);
                                Msg::SessionCreationFailed(error)
                            }
                        }
                    }
                });
            }

            Cmd::AsyncLoadSessions(client) => {
                // Spawn async session loading task
                self.task_manager.spawn_task(async move {
                    match client.list_sessions().await {
                        Ok(sessions) => Msg::SessionsLoaded(sessions),
                        Err(error) => Msg::SessionsLoadFailed(error),
                    }
                });
            }

            Cmd::AsyncLoadSessionMessages(client, session_id) => {
                // Spawn async session messages loading task
                self.task_manager.spawn_task(async move {
                    match client.get_messages(&session_id).await {
                        Ok(messages) => Msg::SessionMessagesLoaded(messages),
                        Err(error) => Msg::SessionMessagesLoadFailed(error),
                    }
                });
            }

            Cmd::AsyncSessionAbort => {
                self.task_manager
                    // TODO eventually call proper API to cancel loop
                    .spawn_task(async move { Msg::ChangeState(AppState::Welcome) });
            }

            Cmd::AsyncCancelTask(task_id) => {
                self.task_manager.cancel_task(task_id);
            }

            Cmd::Batch(commands) => {
                // Handle batch commands by processing them in the main loop
                // rather than recursively to avoid infinite future size
                for cmd in commands {
                    match cmd {
                        Cmd::AsyncSpawnClientDiscovery
                        | Cmd::AsyncSpawnSessionInit(_)
                        | Cmd::AsyncCreateSessionWithMessage(_, _)
                        | Cmd::AsyncLoadSessions(_)
                        | Cmd::AsyncLoadSessionMessages(_, _)
                        | Cmd::AsyncCancelTask(_)
                        | Cmd::RebootTerminalWithInline(_)
                        | Cmd::ResizeInlineViewport(_)
                        | Cmd::AsyncSessionAbort
                        | Cmd::AutoResizeTerminal => {
                            Box::pin(self.spawn_command(cmd)).await?;
                        }
                        Cmd::None => {}
                        Cmd::Batch(_) => {
                            return Err(format!(
                                "Nested Cmd::Batch detected in spawn_command. This indicates a logic error in the update() function. \
                                Batch commands should only contain non-batch commands to avoid infinite recursion and stack overflow. \
                                Please review the update() logic that produced this nested batch."
                            ).into());
                        }
                    }
                }
            }

            Cmd::None => {}
        }
        Ok(())
    }
}
