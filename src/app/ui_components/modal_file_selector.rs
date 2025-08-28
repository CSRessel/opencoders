use crate::app::ui_components::{ModalSelector, ModalSelectorEvent, SelectableData, SelectorConfig, SelectorMode, TableColumn};
use opencode_sdk::models::File;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    text::{Span},
    widgets::{Cell, Widget},
};

/// Data wrapper for file selection
#[derive(Debug, Clone, PartialEq)]
pub struct FileData {
    pub file: File,
}

impl FileData {
    pub fn from_file(file: File) -> Self {
        Self { file }
    }

    fn format_changes(&self) -> Vec<Span<'static>> {
        let mut spans = Vec::new();

        if self.file.added > 0 {
            spans.push(Span::styled(
                format!("+{}", self.file.added),
                Style::default().fg(Color::Green),
            ));
        }

        if self.file.removed > 0 {
            if !spans.is_empty() {
                spans.push(Span::raw(" "));
            }
            spans.push(Span::styled(
                format!("-{}", self.file.removed),
                Style::default().fg(Color::Red),
            ));
        }

        if spans.is_empty() {
            spans.push(Span::raw(""));
        }

        spans
    }
}

impl SelectableData for FileData {
    fn to_cells(&self) -> Vec<Cell> {
        vec![
            Cell::from(ratatui::text::Line::from(self.format_changes())),
            Cell::from(self.file.path.clone()),
        ]
    }

    fn to_string(&self) -> String {
        self.file.path.clone()
    }

    fn to_spans(&self) -> Option<Vec<Span>> {
        let mut spans = self.format_changes();
        spans.push(Span::raw(" "));
        spans.push(Span::raw(&self.file.path));
        Some(spans)
    }
}

/// Submessage enum for the file selector that wraps generic events
#[derive(Debug, Clone, PartialEq)]
pub enum MsgModalFileSelector {
    Event(ModalSelectorEvent<FileData>),
    FileSelected(File),
    Cancel,
}

/// File selector that wraps the generic ModalSelector
#[derive(Debug, Clone)]
pub struct FileSelector {
    pub modal: ModalSelector<FileData>,
}

impl FileSelector {
    pub fn new() -> Self {
        let config = SelectorConfig {
            title: "File Selector".to_string(),
            footer: Some("↑↓ navigate, Enter select, Esc close".to_string()),
            max_width: Some(80),
            max_height: Some(20),
            show_scrollbar: true,
            alternating_rows: false,
            border_color: Color::Blue,
            selected_style: Style::default()
                .add_modifier(Modifier::REVERSED)
                .fg(Color::Blue),
            header_style: Style::default().fg(Color::Yellow),
            row_style: Style::default().fg(Color::White),
            alt_row_style: None,
        };

        let columns = vec![
            TableColumn::new("Changes", Constraint::Length(10)),
            TableColumn::new("File Path", Constraint::Min(20)),
        ];

        Self {
            modal: ModalSelector::new(config, SelectorMode::Table { columns }),
        }
    }

    pub fn set_files(&mut self, files: Vec<File>) {
        let file_data: Vec<FileData> = files.into_iter().map(FileData::from_file).collect();
        self.modal.set_items(file_data);
    }

    // Compatibility methods
    pub fn selected_index(&self) -> Option<usize> {
        self.modal.selected_index()
    }

    pub fn navigate_up(&mut self) {
        self.modal.navigate_up();
    }

    pub fn navigate_down(&mut self) {
        self.modal.navigate_down();
    }

    pub fn get_selected_file(&self) -> Option<&File> {
        self.modal.selected_item().map(|data| &data.file)
    }

    pub fn is_visible(&self) -> bool {
        self.modal.is_visible()
    }

    pub fn show(&mut self) {
        self.modal.show();
    }

    pub fn hide(&mut self) {
        self.modal.hide();
    }
}

impl Widget for &FileSelector {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.modal.render(area, buf);
    }
}
