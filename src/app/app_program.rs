use crate::app::{
    event_msg::{Cmd, Msg},
    event_subscriptions::poll_subscriptions,
    tea_model::{AppState, Model},
    tea_update::update,
    tea_view::view,
    ui_terminal::TerminalGuard,
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;

pub struct Program {
    model: Model,
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    _guard: TerminalGuard,
}

impl Program {
    pub fn new(inline_mode: bool) -> Result<Self, Box<dyn std::error::Error>> {
        let (guard, terminal) = TerminalGuard::new(inline_mode)?;
        let model = Model::new();

        Ok(Program {
            model,
            terminal,
            _guard: guard,
        })
    }

    pub fn run(mut self) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            // View: Pure rendering
            self.terminal.draw(|f| view(&self.model, f))?;

            // Check for quit state
            if matches!(self.model.state, AppState::Quit) {
                break;
            }

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
            Cmd::None => Ok(()),
            // Future: Handle other commands like API calls, file operations, etc.
        }
    }
}

