use crate::app::tea_model::{Model, RepeatShortcutKey};
use crate::app::view_model_context::ViewModelContext;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};
use throbber_widgets_tui::Throbber;

const MODE_COLORS: [Color; 3] = [Color::Black, Color::Magenta, Color::Green];
const MODE_DEFAULT_COLOR: Color = Color::Gray;

#[derive(Debug, Clone, Default)]
pub struct StatusBar;

impl StatusBar {
    pub fn new() -> Self {
        Self
    }
}

impl Widget for &StatusBar {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let model = ViewModelContext::current();

        // Get mode info
        let (mode_text, mode_color) = if let Some(mode_index) = model.get().mode_state {
            let bg_color = MODE_COLORS
                .get(mode_index as usize)
                .copied()
                .unwrap_or(MODE_DEFAULT_COLOR);
            (
                model
                    .get()
                    .get_current_mode_name()
                    .unwrap_or("UNKNOWN".to_string()),
                bg_color,
            )
        } else {
            ("UNKNOWN".to_string(), MODE_DEFAULT_COLOR)
        };

        // Calculate layout sections
        let mut mode_len = mode_text.len();
        let mode_padding = " ".repeat(8 - mode_len);
        mode_len += mode_padding.len();

        let status_text = format!(
            " {} {}", // TODO: (20.4k tokens / 9% context)
            model.get().sdk_provider,
            model.get().sdk_model,
        );
        let status_len = status_text.len();

        // Layout the status bar horizontally
        let start_width = (area.width / 4).min(10);
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Min(start_width / 2),      // Loading section
                Constraint::Min(start_width),          // Session ID section
                Constraint::Length(status_len as u16), // Provider/model section
                Constraint::Length(mode_len as u16),   // Mode section
            ])
            .split(area);

        // Render loading indicator
        let loading_label = match (
            &model.get().has_active_timeout(),
            &model.get().repeat_shortcut_timeout,
            &model.get().active_task_count,
        ) {
            (true, Some(timeout), _) => match timeout.key {
                RepeatShortcutKey::Leader => "Shortcut waiting...",
                RepeatShortcutKey::CtrlC => "Ctrl+C again to confirm",
                RepeatShortcutKey::CtrlD => "Ctrl+D again to confirm",
                RepeatShortcutKey::Esc => "Esc again to confirm",
            },
            (_, _, 0) => "Ready",
            _ => "Working...",
        };

        if !model.get().session_is_idle || model.get().active_task_count > 0 {
            Throbber::default()
                .label(loading_label)
                .render(chunks[0], buf);
        } else {
            Paragraph::new(loading_label).render(chunks[0], buf);
        }

        // Render session ID if present (from model instead of local state)
        if let Some(session_id) = model.get().current_session_id() {
            let session_paragraph = Paragraph::new(Line::from(Span::styled(
                &session_id,
                Style::default().fg(Color::DarkGray),
            )));
            session_paragraph.render(chunks[1], buf);
        }

        // Render provider/model info
        let status_paragraph = Paragraph::new(Line::from(status_text));
        status_paragraph.render(chunks[2], buf);

        // Render mode indicator
        let mode_paragraph = Paragraph::new(Line::from(Span::styled(
            format!(" {}{} ", mode_text, mode_padding),
            Style::default().bg(mode_color).fg(Color::White),
        )));
        mode_paragraph.render(chunks[3], buf);
    }
}
