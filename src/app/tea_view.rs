use crate::app::{
    tea_model::*,
    ui_components::{
        banner::{create_welcome_text, welcome_text_height},
        text_input::TEXT_INPUT_HEIGHT,
        MessageContext, MessageLog, MessageRenderer, PopoverSelector, StatusBar,
    },
    view_model_context::ViewModelContext,
};
use eyre::WrapErr;
use ratatui::{
    backend::CrosstermBackend,
    crossterm,
    layout::{Constraint, Direction, Layout},
    prelude::Widget,
    style::{Color, Style},
    text::{Line, Text, ToText},
    widgets::{Paragraph, Wrap},
    Frame, Terminal,
};
use std::io;

// Config:
// - inline_mode          := true
// - ui_block_is_rounded  := true
// - ui_block_is_bordered := true
// - ui_block_padding     := 0
// - ui_status_is_bottom  := true
// - ui_status_use_labels := true
//
// Design:
//
// ╭─────────────────────────────────────────────────────────────────────────────────────────────╮
// │ >                                                                                           │
// ╰─────────────────────────────────────────────────────────────────────────────────────────────╯
// ⠧ Working                                    Anthropic Claude Opus (21.4k tokens / 9% context)
//
// ^ throbber                                   ^ label provider       ^ count        ^ percent
//   ^ label                                              ^ label model      ^ label     ^ label
//
// Messages:
//
// ╭──────────╮
// │ > /quit  │
// ╰──────────╯

pub fn view_manual(
    model: &Model,
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
) -> crate::app::error::Result<()> {
    match model.state {
        AppState::TextEntry => render_history(model, terminal)?,
        _ => {}
    }
    Ok(())
}

fn render_history(
    model: &Model,
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
) -> crate::app::error::Result<()> {
    let message_containers = model.message_containers_for_rendering();
    let (window_cols, _window_rows) = crossterm::terminal::size()?;

    for container in &message_containers {
        let renderer = MessageRenderer::from_message_container(
            container,
            MessageContext::Inline,
            model.verbosity_level,
        );
        let rendered_text = renderer.render();
        let paragraph = Paragraph::new(rendered_text).wrap(Wrap { trim: false });
        let line_count = paragraph.line_count(window_cols) as u16;

        terminal.insert_before(line_count, |buf| {
            paragraph.render(buf.area, buf);
        })?;
    }

    Ok(())
}

pub fn view(model: &Model, frame: &mut Frame) {
    ViewModelContext::with_model(model, || {
        match &model.state {
            AppState::Welcome => render_welcome_screen(frame),
            AppState::ConnectingToServer => render_connecting_screen(frame),
            AppState::InitializingSession => render_initializing_session_screen(frame),
            AppState::TextEntry => render_text_entry_screen(frame),
            AppState::SelectSession => {
                // Render the underlying state first (Welcome screen)
                render_welcome_screen(frame);
                // Then render the popover selector on top
                frame.render_widget(&model.session_selector, frame.area());
            }
            AppState::ConnectionError(error) => render_error_screen(frame, error),
            AppState::Quit => {} // No rendering needed for quit state
        };
    })
}

pub fn view_clear(frame: &mut Frame) {
    // Write an empty frame to force full redraw of all cells
    frame.render_widget(Paragraph::new(""), frame.area());
}

fn render_welcome_screen(frame: &mut Frame) {
    let model = ViewModelContext::current();
    let status_text = match model.connection_status() {
        ConnectionStatus::SessionReady => "✓ Session ready!",
        ConnectionStatus::ClientReady => "✓ Connected!",
        ConnectionStatus::Connected => "Connected to server...",
        ConnectionStatus::Connecting => "Connecting to OpenCode server...",
        ConnectionStatus::InitializingSession => "Initializing session...",
        ConnectionStatus::Disconnected => "Disconnected from server! Press 'r' to retry",
        ConnectionStatus::Error(ref _error) => "Connection failed! Press 'r' to retry",
    }
    .to_string();
    let help_text = "\n
    Enter    start input
    ^x l     select session
    ^x tab   toggle view
    ^x q     quit
    ";

    let text = Text::from(status_text + help_text);
    let line_height =
        (text.to_text().lines.len().saturating_add(2) as u16).max(model.get().config.height);
    let paragraph = Paragraph::new(text);

    if model.init().inline_mode() {
        let vertical_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(line_height), Constraint::Min(0)])
            .split(frame.area());
        frame.render_widget(paragraph, vertical_chunks[0]);
    } else {
        let constraints = vec![
            Constraint::Length(welcome_text_height().saturating_add(2)),
            Constraint::Length(line_height),
        ];

        let vertical_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(frame.area());

        frame.render_widget(create_welcome_text(), vertical_chunks[0]);
        frame.render_widget(paragraph, vertical_chunks[1]);
    };
}

fn render_text_entry_screen(frame: &mut Frame) {
    let model = ViewModelContext::current();
    let terminal_width = frame.area().width;
    let content_width = match model.init().inline_mode() {
        // Inline is max width 120 for status box
        true => terminal_width.max(120),
        // Full screen is 1 character padding
        false => terminal_width.saturating_sub(2),
    };
    let left_padding = (terminal_width.saturating_sub(content_width)) / 2;
    let right_padding = terminal_width.saturating_sub(content_width.saturating_add(left_padding));

    // Create horizontal layout for centering
    let horizontal_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(left_padding),
            Constraint::Length(content_width),
            Constraint::Length(right_padding),
        ])
        .split(frame.area());

    let content_area = horizontal_chunks[1];

    // Use dynamic height from TextInputArea and add space for StatusBar
    let text_input_height = model.get().text_input_area.current_height();
    let status_bar_height = 1;
    let total_input_section_height = text_input_height + status_bar_height;

    let spacer_height = match model.init().inline_mode() {
        true => &model.get().config.height - total_input_section_height,
        false => 0,
    };

    // Create vertical layout for (optional) message log and input section
    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),                             // (optional) Message log
            Constraint::Length(spacer_height),              // (optional) Buffer space
            Constraint::Length(total_input_section_height), // Input area + status bar
        ])
        .split(content_area);

    // Split the input section into text input and status bar
    let input_section_area = vertical_chunks[2];
    let input_section_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(text_input_height), // Text input area
            Constraint::Length(status_bar_height), // Status bar
        ])
        .split(input_section_area);

    if model.init().inline_mode() {
        // Render only the text input and status bar for inline mode
        frame.render_widget(&model.get().text_input_area, input_section_chunks[0]);
        let status_bar = StatusBar::new();
        frame.render_widget(&status_bar, input_section_chunks[1]);
    } else {
        // Note: We can't send messages from the view layer in TEA architecture
        // Scroll validation will happen during scroll events and when content changes
        frame.render_widget(&model.get().message_log, vertical_chunks[0]);
        frame.render_widget(&model.get().text_input_area, input_section_chunks[0]);
        let status_bar = StatusBar::new();
        frame.render_widget(&status_bar, input_section_chunks[1]);
    }
}

fn render_connecting_screen(frame: &mut Frame) {
    let model = ViewModelContext::current();
    let text = Text::from(vec![
        Line::from("Connecting to OpenCode server..."),
        Line::from(""),
        Line::from("Looking for running OpenCode processes..."),
        Line::from("Press 'q' or 'Esc' to cancel"),
    ]);
    let paragraph = Paragraph::new(text).style(Style::default().fg(Color::Yellow));

    if model.init().inline_mode() {
        let vertical_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(4)])
            .split(frame.area());
        frame.render_widget(paragraph, vertical_chunks[1]);
    } else {
        let vertical_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(4),
                Constraint::Min(0),
            ])
            .split(frame.area());
        frame.render_widget(paragraph, vertical_chunks[1]);
    }
}

fn render_initializing_session_screen(frame: &mut Frame) {
    let model = ViewModelContext::current();
    let client_url = model.client_base_url();

    let text = Text::from(vec![
        Line::from("Initializing session..."),
        Line::from(""),
        Line::from(format!("Connected to: {}", client_url)),
        Line::from("Setting up your coding session..."),
        Line::from("Press 'q' or 'Esc' to cancel"),
    ]);
    let paragraph = Paragraph::new(text).style(Style::default().fg(Color::Blue));

    if model.init().inline_mode() {
        let vertical_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(5)])
            .split(frame.area());
        frame.render_widget(paragraph, vertical_chunks[1]);
    } else {
        let vertical_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(5),
                Constraint::Min(0),
            ])
            .split(frame.area());
        frame.render_widget(paragraph, vertical_chunks[1]);
    }
}

fn render_error_screen(frame: &mut Frame, error: &str) {
    let model = ViewModelContext::current();
    let text = Text::from(vec![
        Line::from("Connection Error"),
        Line::from(""),
        Line::from(error),
        Line::from(""),
        Line::from("Suggestions:"),
        Line::from("• Make sure OpenCode server is running"),
        Line::from("• Check OPENCODE_SERVER_URL environment variable"),
        Line::from("• Try running: opencode serve"),
        Line::from(""),
        Line::from("Press 'r' to retry, 'q' or 'Esc' to quit"),
    ]);
    let paragraph = Paragraph::new(text).style(Style::default().fg(Color::Red));

    if model.init().inline_mode() {
        let vertical_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(10)])
            .split(frame.area());
        frame.render_widget(paragraph, vertical_chunks[1]);
    } else {
        let vertical_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(10),
                Constraint::Min(0),
            ])
            .split(frame.area());
        frame.render_widget(paragraph, vertical_chunks[1]);
    }
}
