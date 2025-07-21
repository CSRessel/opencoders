use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::Paragraph,
    Terminal, TerminalOptions, Viewport,
};
use std::io;

mod components;
use components::{TextInput, text_input::TextInputEvent};

pub struct App {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    text_input: TextInput,
    app_state: AppState,
    last_input: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AppState {
    Welcome,
    TextEntry,
}

const INLINE_MODE: bool = true;

impl App {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let terminal = Self::setup_terminal()?;
        let mut text_input = TextInput::new();
        text_input.set_focus(true);
        
        Ok(App {
            terminal,
            text_input,
            app_state: AppState::Welcome,
            last_input: None,
        })
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            match self.app_state {
                AppState::Welcome => {
                    let (letters, colors) = self.get_opencoders_ascii_art();
                    let colored_text = self.create_colored_ascii_text(&letters, &colors);
                    self.render_welcome_screen(&colored_text)?;
                    
                    if let Event::Key(key) = event::read()? {
                        match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => break,
                            KeyCode::Enter => {
                                self.app_state = AppState::TextEntry;
                            }
                            _ => {}
                        }
                    }
                }
                AppState::TextEntry => {
                    self.render_text_entry_screen()?;
                    
                    if let Event::Key(key) = event::read()? {
                        if let Some(input_event) = self.map_key_to_text_input_event(key) {
                            if let Some(submitted_text) = self.text_input.handle_event(input_event) {
                                self.last_input = Some(submitted_text);
                            }
                        } else {
                            match key.code {
                                KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => break,
                                KeyCode::Esc => {
                                    self.app_state = AppState::Welcome;
                                    self.text_input.clear();
                                    self.last_input = None;
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>, Box<dyn std::error::Error>>
    {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnableMouseCapture)?;

        let backend = CrosstermBackend::new(stdout);
        
        let viewport = if INLINE_MODE {
            Viewport::Inline(10) // Reserve 10 lines for inline mode
        } else {
            Viewport::Fullscreen
        };
        
        let terminal = Terminal::with_options(
            backend,
            TerminalOptions { viewport }
        )?;
        
        Ok(terminal)
    }

    fn get_opencoders_ascii_art(&self) -> (Vec<Vec<&'static str>>, Vec<Color>) {
        #[rustfmt::skip]
        let letters = vec![
            vec!["▄▀▀█",
                 "█░░█",
                 "▀▀▀ "], // o
            vec!["▄▀▀█",
                 "█░░█",
                 "█▀▀ "], // p
            vec!["▄▀▀▀",
                 "█▀▀▀",
                 "▀▀▀▀"], // e
            vec!["█▀▀▄",
                 "█░░█",
                 "▀  ▀"], // n
            vec!["▄▀▀▀",
                 "█░░░",
                 "▀▀▀▀"], // c
            vec!["▄▀▀█",
                 "█░░█",
                 "▀▀▀ "], // o
            vec!["█▀▀▄",
                 "█░░█",
                 "▀▀▀ "], // d
            vec!["▄▀▀▀",
                 "█▀▀▀",
                 "▀▀▀▀"], // e
            vec!["█▀▀█",
                 "█▀▀▄",
                 "▀  ▀"], // r
            vec!["▄▀▀▀",
                 "▀▀▀█",
                 "▀▀▀ "], // s
        ];

        let colors = vec![
            Color::Gray,
            Color::Gray,
            Color::Gray,
            Color::Gray,
            Color::White,
            Color::White,
            Color::White,
            Color::White,
            Color::Gray,
            Color::Gray,
        ];

        (letters, colors)
    }

    fn create_colored_ascii_text<'a>(
        &self,
        letters: &[Vec<&'a str>],
        colors: &[Color],
    ) -> Text<'a> {
        let mut lines = vec![Line::from("")];

        for row in 0..3 {
            let mut spans = Vec::new();

            for (letter_idx, letter) in letters.iter().enumerate() {
                let color = colors.get(letter_idx).unwrap_or(&Color::White);
                let style = Style::default().fg(*color);

                spans.push(Span::styled(letter[row], style));

                if letter_idx < letters.len() - 1 {
                    spans.push(Span::raw(" "));
                }
            }

            lines.push(Line::from(spans));
        }

        lines.push(Line::from(""));
        lines.push(Line::from("Press Enter to start text input, 'q' or 'Esc' to exit..."));
        Text::from(lines)
    }

    fn map_key_to_text_input_event(&self, key: crossterm::event::KeyEvent) -> Option<TextInputEvent> {
        match key.code {
            KeyCode::Char(c) => Some(TextInputEvent::Insert(c)),
            KeyCode::Backspace => Some(TextInputEvent::Delete),
            KeyCode::Enter => Some(TextInputEvent::Submit),
            _ => None,
        }
    }

    fn render_text_entry_screen(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(0),
                    Constraint::Length(3),
                ])
                .split(f.area());

            // Main content area
            if let Some(ref last_input) = self.last_input {
                let response_text = format!("You entered: {}", last_input);
                let paragraph = Paragraph::new(response_text);
                f.render_widget(paragraph, chunks[0]);
            }

            // Text input at bottom
            f.render_widget(&self.text_input, chunks[1]);
        })?;
        Ok(())
    }



    fn render_welcome_screen(&mut self, text: &Text) -> Result<(), Box<dyn std::error::Error>> {
        self.terminal.draw(|f| {
            let paragraph = Paragraph::new(text.clone());
            f.render_widget(paragraph, f.area());
        })?;
        Ok(())
    }


}

impl Drop for App {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(self.terminal.backend_mut(), DisableMouseCapture);
        let _ = self.terminal.show_cursor();
    }
}
