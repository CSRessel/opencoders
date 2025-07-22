use crate::app::{
    event_msg::{Cmd, Msg},
    event_subscriptions::poll_subscriptions,
    tea_model::{AppState, Model},
    tea_update::update,
    tea_view::{view, view_clear, view_manual},
    ui_components::{banner::create_welcome_text, terminal::TerminalGuard},
};
use ratatui::{backend::CrosstermBackend, style::Color, text::Text, Terminal};
use std::io::{self, Write};

pub struct Program {
    model: Model,
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    _guard: TerminalGuard,
}

impl Program {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let model = Model::new();
        if model.inline_mode {
            // Print welcome message to stdout before entering TUI in inline mode
            let welcome_text = create_welcome_text();
            print!("{}", welcome_text);
        }
        let (guard, terminal) = TerminalGuard::new(&model)?;

        Ok(Program {
            model,
            terminal,
            _guard: guard,
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
                // Clear the TUI
                self.terminal.draw(|f| view_clear(&self.model, f))?;
                // Manually execute with crossterm
                view_manual(&self.model)?;
            }

            // View: Pure rendering, within the TUI
            self.terminal.draw(|f| view(&self.model, f))?;

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
            Cmd::None => Ok(()),
            // Future: Handle other commands like API calls, file operations, etc.
        }
    }
}
