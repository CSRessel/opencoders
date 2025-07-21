use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::Paragraph,
    Terminal,
};
use std::io;

pub struct App {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
}

impl App {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let terminal = Self::setup_terminal()?;
        Ok(App { terminal })
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let (letters, colors) = self.get_opencoders_ascii_art();
        let colored_text = self.create_colored_ascii_text(&letters, &colors);
        self.render_welcome_screen(&colored_text)?;
        self.wait_for_exit()?;
        Ok(())
    }

    fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>, Box<dyn std::error::Error>>
    {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        Ok(terminal)
    }

    fn get_opencoders_ascii_art(&self) -> (Vec<Vec<&'static str>>, Vec<Color>) {
        #[rustfmt::skip]
        let letters = vec![
            vec!["▄▀▀▄",
                 "█░░█",
                 " ▀▀ "], // o
            vec!["▄▀▀█",
                 "█░░█",
                 "█▀▀ "], // p
            vec!["▄▀▀▀",
                 "█▀▀▀",
                 " ▀▀▀"], // e
            vec!["█▀▀▄",
                 "█░░█",
                 "▀  ▀"], // n
            vec!["▄▀▀▀",
                 "█░░░",
                 " ▀▀▀"], // c
            vec!["▄▀▀▄",
                 "█░░█",
                 " ▀▀ "], // o
            vec!["█▀▀▄",
                 "█░░█",
                 "▀▀▀ "], // d
            vec!["▄▀▀▀",
                 "█▀▀▀",
                 " ▀▀▀"], // e
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
        Text::from(lines)
    }

    fn wait_for_exit(&self) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc {
                    break;
                }
            }
        }
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
        let _ = execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        );
        let _ = self.terminal.show_cursor();
    }
}

