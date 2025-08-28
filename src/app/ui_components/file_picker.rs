use crate::app::view_model_context::ViewModelContext;
use opencode_sdk::models::File;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Row, Table, TableState, Widget},
};

#[derive(Debug, Clone)]
pub struct FilePicker {
    state: TableState,
}

impl FilePicker {
    pub fn new() -> Self {
        let mut state = TableState::default();
        state.select(Some(0));
        Self { state }
    }

    pub fn selected_index(&self) -> Option<usize> {
        self.state.selected()
    }

    pub fn navigate_up(&mut self, files: &[File]) {
        if files.is_empty() {
            return;
        }

        let current = self.state.selected().unwrap_or(0);
        let new_index = if current == 0 {
            files.len() - 1
        } else {
            current - 1
        };
        self.state.select(Some(new_index));
    }

    pub fn navigate_down(&mut self, files: &[File]) {
        if files.is_empty() {
            return;
        }

        let current = self.state.selected().unwrap_or(0);
        let new_index = if current >= files.len() - 1 {
            0
        } else {
            current + 1
        };
        self.state.select(Some(new_index));
    }

    pub fn get_selected_file<'a>(&self, files: &'a [File]) -> Option<&'a File> {
        self.selected_index().and_then(|i| files.get(i))
    }

    fn format_changes(&self, file: &File) -> Vec<Span<'static>> {
        let mut spans = Vec::new();
        
        if file.added > 0 {
            spans.push(Span::styled(
                format!("+{}", file.added),
                Style::default().fg(Color::Green),
            ));
        }
        
        if file.removed > 0 {
            if !spans.is_empty() {
                spans.push(Span::raw(" "));
            }
            spans.push(Span::styled(
                format!("-{}", file.removed),
                Style::default().fg(Color::Red),
            ));
        }

        if spans.is_empty() {
            spans.push(Span::raw(""));
        }

        spans
    }
}

impl Widget for &FilePicker {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let model = ViewModelContext::current();
        let files = &model.get().file_status;

        if files.is_empty() {
            let block = Block::default()
                .title("File Picker")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue));

            let empty_table = Table::new(
                [Row::new([Cell::from("No files found")])],
                [Constraint::Percentage(100)],
            )
            .block(block);

            empty_table.render(area, buf);
            return;
        }

        let header = Row::new([
            Cell::from("Changes"),
            Cell::from("File Path"),
        ])
        .style(Style::default().fg(Color::Yellow))
        .height(1);

        let rows = files.iter().map(|file| {
            Row::new([
                Cell::from(Line::from(self.format_changes(file))),
                Cell::from(file.path.clone()),
            ])
            .height(1)
        });

        let selected_style = Style::default()
            .add_modifier(Modifier::REVERSED)
            .fg(Color::Blue);

        let block = Block::default()
            .title("File Picker (↑↓ navigate, Enter select, Esc close)")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Blue));

        let table = Table::new(
            rows,
            [
                Constraint::Length(10), // Changes column (narrow)
                Constraint::Min(20),    // File path column (wide)
            ],
        )
        .header(header)
        .block(block)
        .row_highlight_style(selected_style);

        // Need to render with mutable state, so we need to cast
        let mut mutable_self = self.clone();
        ratatui::widgets::StatefulWidget::render(table, area, buf, &mut mutable_self.state);
    }
}