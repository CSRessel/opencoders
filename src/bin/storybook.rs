//! Storybook for UI components - renders components inline and exits
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use opencoders::storybook::stories::text_input_story::TextInputStory;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Terminal, TerminalOptions, Viewport,
};
use std::io;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize terminal for inline rendering
    enable_raw_mode()?;
    let stdout = io::stdout();
    // execute!(stdout, EnterAlternateScreen)?;
    let min_height = 20;
    let viewport = if let Ok((_cols, rows)) = crossterm::terminal::size() {
        Viewport::Inline(rows.max(min_height))
    } else {
        Viewport::Inline(min_height)
    };
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::with_options(backend, TerminalOptions { viewport })?;
    terminal.clear()?;
    terminal.hide_cursor()?;

    // Create text input story
    let mut story = TextInputStory::new();
    let variants = story.variants.clone();

    // Render all variants of the component inline
    terminal.draw(|frame| {
        let area = frame.area();

        // Create layout for title and variants
        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Min(0),    // Variants
            ])
            .split(area);

        // Render title
        let title = Paragraph::new("TextInputArea Component Storybook")
            .block(Block::default().borders(Borders::ALL).title("Storybook"))
            .style(Style::default().fg(Color::Yellow));
        frame.render_widget(title, main_layout[0]);

        // Create layout for variants
        let variants_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                variants
                    .iter()
                    .map(|_| Constraint::Length(8))
                    .collect::<Vec<_>>(),
            )
            .split(main_layout[1]);

        // Render each variant
        for (i, variant) in variants.iter().enumerate() {
            if i < variants_layout.len() {
                let variant_area = variants_layout[i];

                // Create sub-layout for variant title and component
                let variant_layout = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(1), // Variant name
                        Constraint::Length(6), // Component area
                    ])
                    .split(variant_area);

                // Render variant name
                let variant_name = Paragraph::new(Line::from(vec![
                    Span::styled("Variant: ", Style::default().fg(Color::Cyan)),
                    Span::styled(
                        story.get_variant_name(variant),
                        Style::default().fg(Color::White),
                    ),
                ]));
                frame.render_widget(variant_name, variant_layout[0]);

                // Render the component variant
                let component_area = Rect {
                    x: variant_layout[1].x + 2,
                    y: variant_layout[1].y,
                    width: variant_layout[1].width.saturating_sub(4),
                    height: variant_layout[1].height,
                };

                story.render_variant(variant, component_area, frame.buffer_mut());
            }
        }
    })?;

    // Wait briefly to see the output
    std::thread::sleep(std::time::Duration::from_millis(10));

    // Cleanup and exit
    disable_raw_mode()?;
    // execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    println!("");
    println!("TextInputArea storybook rendered successfully!");
    println!("Displayed variants: Empty, With Placeholder, Focused, With Content");
    Ok(())
}

