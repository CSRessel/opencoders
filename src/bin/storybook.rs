//! Storybook for UI components - renders components inline and exits
use color_eyre::Result;
use opencoders::{
    app::{
        tea_model::ModelInit,
        terminal::{init_terminal, restore_terminal},
    },
    storybook::stories::text_input_story::{StoryVariant, TextInputStory},
};
use ratatui::{
    backend::CrosstermBackend,
    crossterm::event::{self, Event, KeyEventKind},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};
use std::io;

fn main() -> Result<()> {
    let init = ModelInit::new(true);
    let min_height = 30;
    let viewport_height = if let Ok((_cols, rows)) = crossterm::terminal::size() {
        rows.max(min_height)
    } else {
        min_height
    };

    let mut terminal = init_terminal(&init, viewport_height)?;
    let app_result = run(&mut terminal);
    restore_terminal(&init)?;
    app_result
}

fn run(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    // Create text input story
    let mut story = TextInputStory::new();
    let variants = story.variants.clone();

    let mut should_quit = false;
    while !should_quit {
        terminal.draw(|frame| {
            render_storybook(frame, &mut story, &variants);
        })?;
        should_quit = handle_events()?;
    }
    Ok(())
}

fn handle_events() -> Result<bool> {
    if let Event::Key(key) = event::read()? {
        if key.kind == KeyEventKind::Press {
            return Ok(true); // Any key press exits
        }
    }
    Ok(false)
}

fn render_storybook(frame: &mut Frame, story: &mut TextInputStory, variants: &[StoryVariant]) {
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
}
