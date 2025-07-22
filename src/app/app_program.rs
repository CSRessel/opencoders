use crate::app::{
    event_msg::{Cmd, Msg},
    event_subscriptions::poll_subscriptions,
    tea_model::{AppState, Model, ModelInit},
    tea_update::update,
    tea_view::{view, view_clear, view_manual},
    ui_components::{banner::create_welcome_text, terminal::TerminalGuard},
};
use ratatui::{backend::CrosstermBackend, style::Color, text::Text, Terminal};
use std::io::{self, Write};

pub struct Program {
    model: Model,
    terminal: Option<Terminal<CrosstermBackend<io::Stdout>>>,
    guard: Option<TerminalGuard>,
}

impl Program {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let model = Model::new();

        // Print welcome message to stdout before entering TUI
        let welcome_text = create_welcome_text();
        print!("{}\n\n", welcome_text);

        let (guard, terminal) = TerminalGuard::new(&model.init)?;

        Ok(Program {
            model,
            terminal: Some(terminal),
            guard: Some(guard),
        })
    }

    pub fn run(mut self) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            // Check for quit state
            if matches!(self.model.state, AppState::Quit) {
                break;
            }

            // View: Manual rendering outside the TUI viewport
            if self.model.needs_manual_output() {
                if let Some(terminal) = self.terminal.as_mut() {
                    // Clear the TUI
                    terminal.draw(|f| view_clear(&self.model, f))?;
                }
                // Manually execute with crossterm
                view_manual(&self.model)?;
            }

            // View: Pure rendering, within the TUI
            if let Some(terminal) = self.terminal.as_mut() {
                terminal.draw(|f| view(&self.model, f))?;
            }

            // Update the model for all consumed state
            // TODO: move to Msg::ChangeState()
            self.model.consume_viewed_state();

            // Subscriptions: Convert external events to messages
            if let Some(msg) = poll_subscriptions(&self.model)? {
                // Update: Pure state transition
                let (new_model, cmd) = update(self.model, msg);
                self.model = new_model;

                // Commands: Execute side effects
                self.execute_command(cmd)?;
            }
        }
        Ok(())
    }

    fn execute_command(&mut self, cmd: Cmd) -> Result<(), Box<dyn std::error::Error>> {
        match cmd {
            Cmd::RebootTerminalWithInline(inline_mode) => {
                // Deconstruct the old terminal by taking ownership from the Option
                let old_guard = self.guard.take();
                let old_terminal = self.terminal.take();

                // Explicitly drop the old guard and terminal
                drop(old_guard);
                drop(old_terminal);

                let new_init = ModelInit::new(self.model.init.height(), inline_mode);
                let (guard, terminal) = TerminalGuard::new(&new_init)?;
                self.guard = Some(guard);
                self.terminal = Some(terminal);
                self.model.init = new_init;
                Ok(())
            }
            Cmd::None => Ok(()),
            // Future: Handle other commands like API calls, file operations, etc.
        }
    }
}
