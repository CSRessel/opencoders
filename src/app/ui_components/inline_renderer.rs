use owo_colors::OwoColorize;
use ratatui::{
    style::Color,
    text::{Line, Span, Text},
};

/// Renders a ratatui::Span with color to a colorized string
fn render_span(span: &Span<'_>) -> String {
    match span.style.fg {
        Some(Color::Reset) => span.content.to_string(),
        Some(Color::Black) => span.content.black().to_string(),
        Some(Color::Red) => span.content.red().to_string(),
        Some(Color::Green) => span.content.green().to_string(),
        Some(Color::Yellow) => span.content.yellow().to_string(),
        Some(Color::Blue) => span.content.blue().to_string(),
        Some(Color::Magenta) => span.content.magenta().to_string(),
        Some(Color::Cyan) => span.content.cyan().to_string(),
        Some(Color::Gray) => span.content.bright_black().to_string(),
        Some(Color::DarkGray) => span.content.black().to_string(),
        Some(Color::LightRed) => span.content.bright_red().to_string(),
        Some(Color::LightGreen) => span.content.bright_green().to_string(),
        Some(Color::LightYellow) => span.content.bright_yellow().to_string(),
        Some(Color::LightBlue) => span.content.bright_blue().to_string(),
        Some(Color::LightMagenta) => span.content.bright_magenta().to_string(),
        Some(Color::LightCyan) => span.content.bright_cyan().to_string(),
        Some(Color::White) => span.content.white().to_string(),
        Some(Color::Rgb(r, g, b)) => span.content.truecolor(r, g, b).to_string(),
        Some(Color::Indexed(_)) => span.content.to_string(), // Fallback for indexed colors
        None => span.content.to_string(),
    }
}

/// Renders a ratatui::Line to a colorized string
fn render_line(line: &Line<'_>) -> String {
    line.spans.iter().map(render_span).collect::<String>()
}

/// Renders ratatui::Text to a colorized string suitable for println!
pub fn render_text(text: &Text<'_>) -> String {
    text.lines
        .iter()
        .map(render_line)
        .collect::<Vec<String>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::style::Style;

    #[test]
    fn test_render_plain_text() {
        let text = Text::from("Hello, world!");
        let result = render_text(&text);
        assert_eq!(result, "Hello, world!");
    }

    #[test]
    fn test_render_colored_span() {
        let span = Span::styled("Colored text", Style::default().fg(Color::Red));
        let line = Line::from(vec![span]);
        let text = Text::from(vec![line]);
        
        let result = render_text(&text);
        // Result should contain ANSI escape codes for red color
        assert!(result.contains("Colored text"));
        assert!(result.len() > "Colored text".len()); // Should have escape codes
    }

    #[test]
    fn test_render_mixed_spans() {
        let line = Line::from(vec![
            Span::styled("Red", Style::default().fg(Color::Red)),
            Span::raw(" and "),
            Span::styled("Blue", Style::default().fg(Color::Blue)),
        ]);
        let text = Text::from(vec![line]);
        
        let result = render_text(&text);
        assert!(result.contains("Red"));
        assert!(result.contains(" and "));
        assert!(result.contains("Blue"));
    }

    #[test]
    fn test_render_multiple_lines() {
        let text = Text::from(vec![
            Line::from("First line"),
            Line::from("Second line"),
        ]);
        
        let result = render_text(&text);
        assert_eq!(result, "First line\nSecond line");
    }

    #[test]
    fn test_rgb_color_rendering() {
        let span = Span::styled("RGB text", Style::default().fg(Color::Rgb(255, 0, 0)));
        let result = render_span(&span);
        assert!(result.contains("RGB text"));
        assert!(result.len() > "RGB text".len()); // Should have escape codes
    }
}