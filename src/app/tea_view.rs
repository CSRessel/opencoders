use crate::app::{
    tea_model::*,
    ui_components::{
        banner::{create_welcome_text, welcome_text_height},
        message_part::StepRenderingMode,
        text_input::TEXT_INPUT_HEIGHT,
        Block, MessageContext, MessageLog, MessageRenderer, Paragraph, SessionSelector, StatusBar,
    },
    view_model_context::ViewModelContext,
};
use eyre::WrapErr;
use ratatui::{
    backend::CrosstermBackend,
    crossterm,
    layout::{Constraint, Direction, Layout, Rect},
    prelude::Widget,
    style::{Color, Style},
    text::{Line, Text, ToText},
    widgets::{Borders, Wrap},
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

pub fn render_manual_inline_history(
    model: &Model,
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
) -> crate::app::error::Result<()> {
    let message_containers = model.message_containers_for_rendering();
    let (window_cols, _window_rows) = crossterm::terminal::size()?;

    for container in &message_containers {
        let renderer =
            MessageRenderer::step_safe(container, MessageContext::Inline, model.verbosity_level);
        let rendered_text = renderer.render();
        let paragraph = Paragraph::new(rendered_text).wrap(Wrap { trim: false });
        let line_count = paragraph.clone().line_count(window_cols) as u16;

        terminal.insert_before(line_count, |buf| {
            paragraph.render(buf.area, buf);
        })?;
    }

    Ok(())
}

pub fn view(model: &Model, frame: &mut Frame) {
    ViewModelContext::with_model(model, || {
        if model.is_connnection_modal_active() {
            render_connecting_screen(frame, frame.area());
        } else {
            // First render the text entry
            render_base_screen(frame);

            // Then render the modals depending on state
            match &model.state {
                AppModalState::ModalSessionSelect => {
                    // Then render the popover selector on top
                    frame.render_widget(&model.modal_session_selector, frame.area());
                }
                AppModalState::ModalHelp => frame.render_widget(
                    Paragraph::new("help!!!!!")
                        .block(Block::default().borders(Borders::ALL).title("Help")),
                    frame.area(),
                ),
                // No modals/overlays/notifications needed
                _ => {}
            };
        }
    })
}

pub fn view_clear(frame: &mut Frame) {
    // Write an empty frame to force full redraw of all cells
    frame.render_widget(Paragraph::new(""), frame.area());
}

fn render_base_screen(frame: &mut Frame) {
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
            Constraint::Length(total_input_section_height), // Input textarea + status bar
        ])
        .split(content_area);
    let fullscreen_chunk = vertical_chunks[0];
    let spacer_chunk = vertical_chunks[1];
    let input_chunk = vertical_chunks[2];

    // Split the input section into textarea and status bar
    let input_section_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(text_input_height), // Textarea
            Constraint::Length(status_bar_height), // Status bar
        ])
        .split(input_chunk);
    let input_textarea = input_section_chunks[0];
    let input_status = input_section_chunks[1];

    if model.init().inline_mode() {
        // Render file selector on top of spacer_chunk
        if matches!(model.state(), AppModalState::ModalFileSelect) {
            frame.render_widget(&model.get().modal_file_selector, spacer_chunk);
        } else {
            render_main_body(frame, spacer_chunk);
        }
        frame.render_widget(&model.get().text_input_area, input_textarea);
        let status_bar = StatusBar::new();
        frame.render_widget(&status_bar, input_status);
    } else {
        // In fullscreen mode, we have more space - render file selector above the text input
        if matches!(model.state(), AppModalState::ModalFileSelect) {
            // Split fullscreen area to accommodate file selector
            let fullscreen_with_selector_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(0),     // Message log
                    Constraint::Length(10), // File selector (fixed height)
                ])
                .split(fullscreen_chunk);

            render_main_body(frame, fullscreen_with_selector_chunks[0]);
            frame.render_widget(
                &model.get().modal_file_selector,
                fullscreen_with_selector_chunks[1],
            );
        } else {
            render_main_body(frame, fullscreen_chunk);
        }

        frame.render_widget(&model.get().text_input_area, input_textarea);
        let status_bar = StatusBar::new();
        frame.render_widget(&status_bar, input_status);
    }
}

fn render_main_body(frame: &mut Frame, buf: Rect) {
    let model = ViewModelContext::current();

    if model.get().is_session_ready() {
        if !model.init().inline_mode() {
            frame.render_widget(&model.get().message_log, buf);
        }
    } else {
        let help_text = "\n
    ^x l     select session
    ^x tab   toggle view
    ^x q     quit
    ";

        let welcome_text = Text::from(format!("\n{}{}", model.connection_status(), help_text));
        let line_height = (welcome_text.to_text().lines.len().saturating_add(2) as u16)
            .max(model.get().config.height);
        let paragraph = Paragraph::new(welcome_text);

        frame.render_widget(paragraph, buf);
    }
}

fn render_connecting_screen(frame: &mut Frame, rect: Rect) {
    let model = ViewModelContext::current();
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Connection Status");
    let paragraph = match &model.get().state {
        AppModalState::Connecting(ConnectionStatus::Connecting) => {
            let text = Text::from(vec![
                Line::from("Connecting to OpenCode server..."),
                Line::from(""),
                Line::from("Looking for running OpenCode processes..."),
                Line::from(""),
                Line::from("Press 'q' or 'Esc' to cancel"),
            ]);
            Paragraph::new(text)
                .style(Style::default().fg(Color::Yellow))
                .block(block)
        }
        AppModalState::Connecting(ConnectionStatus::InitializingSession) => {
            let client_url = model.client_base_url();
            let text = Text::from(vec![
                Line::from("Initializing session..."),
                Line::from(""),
                Line::from(format!("Connected to: {}", client_url)),
                Line::from("Setting up your coding session..."),
                Line::from("Press 'q' or 'Esc' to cancel"),
            ]);
            Paragraph::new(text)
                .style(Style::default().fg(Color::Blue))
                .block(block)
        }
        AppModalState::Connecting(ConnectionStatus::Error(error)) => {
            let text = Text::from(vec![
                Line::from(format!("Connection Error! {}", error.to_string())),
                Line::from("Suggestions:"),
                Line::from("• Make sure OpenCode server is running"),
                Line::from("• Check OPENCODE_SERVER_URL environment variable"),
                Line::from("Press 'r' to retry, 'q' or 'Esc' to quit"),
            ]);
            Paragraph::new(text)
                .style(Style::default().fg(Color::Red))
                .block(block)
        }
        _ => Paragraph::new(""),
    };

    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(5),
            Constraint::Min(0),
        ])
        .split(frame.area());
    frame.render_widget(paragraph, vertical_chunks[1]);
    // }
}
