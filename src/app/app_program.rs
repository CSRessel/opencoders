use crate::app::{
    event_msg::{Cmd, Msg},
    event_subscriptions::poll_subscriptions,
    tea_model::{AppState, Model},
    tea_update::update,
    tea_view::view,
    ui_terminal::TerminalGuard,
};
use ratatui::{backend::CrosstermBackend, style::Color, Terminal};
use std::io::{self, Write};

pub struct Program {
    model: Model,
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    _guard: TerminalGuard,
    inline_mode: bool,
}

impl Program {
    pub fn new(inline_mode: bool) -> Result<Self, Box<dyn std::error::Error>> {
        // Print welcome message to stdout before entering TUI in inline mode
        if inline_mode {
            Self::print_welcome_to_stdout()?;
        }

        let (guard, terminal) = TerminalGuard::new(inline_mode)?;
        let model = Model::new();

        Ok(Program {
            model,
            terminal,
            _guard: guard,
            inline_mode,
        })
    }

    pub fn run(mut self) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            // In inline mode, print overflow messages to stdout before rendering
            if self.inline_mode {
                self.print_overflow_messages_to_stdout()?;
            }

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

    fn print_overflow_messages_to_stdout(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let messages_to_print = self.model.messages_needing_stdout_print();

        if !messages_to_print.is_empty() {
            // Clear the current TUI display first
            self.terminal.clear()?;

            // Temporarily disable raw mode to print to stdout
            crossterm::terminal::disable_raw_mode()?;

            for message in messages_to_print {
                // Use crossterm to apply grey color for the > prefix
                use crossterm::{
                    execute,
                    style::{Print, ResetColor, SetForegroundColor},
                };
                let mut stdout = io::stdout();

                execute!(
                    stdout,
                    SetForegroundColor(crossterm::style::Color::DarkGrey),
                    Print("> "),
                    ResetColor,
                    Print(message),
                    Print("\r\n")
                )?;
                stdout.flush()?;
            }

            // Re-enable raw mode
            crossterm::terminal::enable_raw_mode()?;

            // Mark these messages as printed
            self.model
                .mark_messages_printed_to_stdout(messages_to_print.len());
        }

        Ok(())
    }

    fn print_welcome_to_stdout() -> Result<(), Box<dyn std::error::Error>> {
        let welcome_text = Self::create_welcome_text();
        println!("{}", welcome_text);
        Ok(())
    }

    fn create_welcome_text() -> String {
        let letters = vec![
            vec!["▄▀▀█", "█░░█", "▀▀▀ "], // o
            vec!["▄▀▀█", "█░░█", "█▀▀ "], // p
            vec!["▄▀▀▀", "█▀▀▀", "▀▀▀▀"], // e
            vec!["█▀▀▄", "█░░█", "▀  ▀"], // n
            vec!["▄▀▀▀", "█░░░", "▀▀▀▀"], // c
            vec!["▄▀▀█", "█░░█", "▀▀▀ "], // o
            vec!["█▀▀▄", "█░░█", "▀▀▀ "], // d
            vec!["▄▀▀▀", "█▀▀▀", "▀▀▀▀"], // e
            vec!["█▀▀█", "█▀▀▄", "▀  ▀"], // r
            vec!["▄▀▀▀", "▀▀▀█", "▀▀▀ "], // s
        ];

        let mut lines = vec![String::new()];

        for row in 0..3 {
            let mut line = String::new();
            for (letter_idx, letter) in letters.iter().enumerate() {
                line.push_str(letter[row]);
                if letter_idx < letters.len() - 1 {
                    line.push(' ');
                }
            }
            lines.push(line);
        }

        lines.join("\n")
    }

    fn execute_command(&mut self, cmd: Cmd) -> Result<(), Box<dyn std::error::Error>> {
        match cmd {
            Cmd::None => Ok(()),
            // Future: Handle other commands like API calls, file operations, etc.
        }
    }
}
