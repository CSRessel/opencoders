mod sdk;

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

pub fn trim_lines_leading(s: &str) -> String {
    s.lines()
        .map(|line| line.trim_start().trim_end())
        .collect::<Vec<&str>>()
        .join("\n")
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>, Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

fn cleanup_terminal(
    mut terminal: Terminal<CrosstermBackend<io::Stdout>>,
) -> Result<(), Box<dyn std::error::Error>> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

fn get_opencoders_ascii_art() -> (Vec<Vec<&'static str>>, Vec<Color>) {
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

fn create_colored_ascii_text<'a>(letters: &[Vec<&'a str>], colors: &[Color]) -> Text<'a> {
    let mut lines = vec![Line::from("")]; // Empty line at top

    // For each row (0, 1, 2)
    for row in 0..3 {
        let mut spans = Vec::new();

        // For each letter
        for (letter_idx, letter) in letters.iter().enumerate() {
            let color = colors.get(letter_idx).unwrap_or(&Color::White);
            let style = Style::default().fg(*color);

            // Add the character slice for this row
            spans.push(Span::styled(letter[row], style));

            // Add space between letters (optional)
            if letter_idx < letters.len() - 1 {
                spans.push(Span::raw(" "));
            }
        }

        lines.push(Line::from(spans));
    }

    lines.push(Line::from("")); // Empty line at bottom
    Text::from(lines)
}

fn wait_for_exit() -> Result<(), Box<dyn std::error::Error>> {
    loop {
        if let Event::Key(key) = event::read()? {
            if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc {
                break;
            }
        }
    }
    Ok(())
}

fn render_welcome_screen(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    text: &Text,
) -> Result<(), Box<dyn std::error::Error>> {
    terminal.draw(|f| {
        let paragraph = Paragraph::new(text.clone());
        f.render_widget(paragraph, f.area());
    })?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (letters, colors) = get_opencoders_ascii_art();
    let mut terminal = setup_terminal()?;

    let colored_text = create_colored_ascii_text(&letters, &colors);
    render_welcome_screen(&mut terminal, &colored_text)?;

    wait_for_exit()?;
    cleanup_terminal(terminal)?;

    Ok(())
}
