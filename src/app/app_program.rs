use crate::app::{
    event_msg::{Cmd, Msg},
    event_subscriptions::poll_subscriptions,
    tea_model::{AppState, Model},
    tea_update::update,
    tea_view::{view, view_prefix_inline},
    ui_components::banner::create_welcome_text,
    ui_components::terminal::TerminalGuard,
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
            Self::print_welcome_to_stdout()?;
        }
        let (guard, terminal) = TerminalGuard::new(model.inline_mode)?;

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

            if self.model.inline_mode {
                self.terminal
                    .draw(|f| view_prefix_inline(&self.model, f).unwrap());
            }
            // View: Pure rendering
            self.terminal.draw(|f| view(&self.model, f))?;

            // Mark viewed messages as printed
            self.model
                .mark_messages_printed_to_stdout(self.model.messages_needing_stdout_print().len());

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

    fn print_welcome_to_stdout() -> Result<(), Box<dyn std::error::Error>> {
        let welcome_text = create_welcome_text();
        print!("{}", welcome_text);
        Ok(())
    }

    fn execute_command(&mut self, cmd: Cmd) -> Result<(), Box<dyn std::error::Error>> {
        match cmd {
            Cmd::None => Ok(()),
            // Future: Handle other commands like API calls, file operations, etc.
        }
    }
}
