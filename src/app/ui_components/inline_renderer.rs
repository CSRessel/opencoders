use owo_colors::OwoColorize;
use ratatui::{
    style::Color,
    text::{Line, Span, Text},
};

/// Renders a ratatui::Span to a colorized string
fn render_span(span: &Span) -> String {
    let content = span.content.as_ref();

    // Apply foreground color if present
    if let Some(fg_color) = span.style.fg {
        match fg_color {
            // Named colors
            Color::Reset => content.to_string(),
            Color::Black => content.black().to_string(),
            Color::Red => content.red().to_string(),
            Color::Green => content.green().to_string(),
            Color::Yellow => content.yellow().to_string(),
            Color::Blue => content.blue().to_string(),
            Color::Magenta => content.magenta().to_string(),
            Color::Cyan => content.cyan().to_string(),
            Color::Gray => content.bright_black().to_string(),
            Color::DarkGray => content.black().to_string(),
            Color::LightRed => content.bright_red().to_string(),
            Color::LightGreen => content.bright_green().to_string(),
            Color::LightYellow => content.bright_yellow().to_string(),
            Color::LightBlue => content.bright_blue().to_string(),
            Color::LightMagenta => content.bright_magenta().to_string(),
            Color::LightCyan => content.bright_cyan().to_string(),
            Color::White => content.white().to_string(),

            // RGB colors
            Color::Rgb(r, g, b) => content.truecolor(r, g, b).to_string(),

            // Indexed colors - map to closest equivalent
            Color::Indexed(index) => match index {
                0 => content.black().to_string(),
                1 => content.red().to_string(),
                2 => content.green().to_string(),
                3 => content.yellow().to_string(),
                4 => content.blue().to_string(),
                5 => content.magenta().to_string(),
                6 => content.cyan().to_string(),
                7 => content.white().to_string(),
                8 => content.bright_black().to_string(),
                9 => content.bright_red().to_string(),
                10 => content.bright_green().to_string(),
                11 => content.bright_yellow().to_string(),
                12 => content.bright_blue().to_string(),
                13 => content.bright_magenta().to_string(),
                14 => content.bright_cyan().to_string(),
                15 => content.bright_white().to_string(),
                _ => content.white().to_string(), // Default for higher indexed colors
            },
        }
    } else {
        content.to_string()
    }
}

/// Renders a ratatui::Line to a colorized string
fn render_line(line: &Line) -> String {
    line.spans
        .iter()
        .map(render_span)
        .collect::<Vec<_>>()
        .join("")
}

/// Renders ratatui::Text to colorized terminal output
pub fn render_text_inline(text: &Text) -> String {
    text.lines
        .iter()
        .map(render_line)
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::style::{Color, Style};

    #[test]
    fn test_render_simple_text() {
        let text = Text::from("Hello, World!");
        let result = render_text_inline(&text);
        assert_eq!(result, "Hello, World!");
    }

    #[test]
    fn test_render_colored_span() {
        let span = Span::styled("Colored text", Style::default().fg(Color::Red));
        let line = Line::from(vec![span]);
        let text = Text::from(vec![line]);
        let result = render_text_inline(&text);
        // The result should contain ANSI escape codes for red color
        assert!(result.contains("Colored text"));
    }

    #[test]
    fn test_render_mixed_colors() {
        let line = Line::from(vec![
            Span::styled("Red ", Style::default().fg(Color::Red)),
            Span::styled("Green ", Style::default().fg(Color::Green)),
            Span::raw("Plain"),
        ]);
        let text = Text::from(vec![line]);
        let result = render_text_inline(&text);
        assert!(result.contains("Red"));
        assert!(result.contains("Green"));
        assert!(result.contains("Plain"));
    }
}

