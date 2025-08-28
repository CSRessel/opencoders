use std::u16;

use crate::app::{
    event_msg::{Cmd, CmdOrBatch},
    tea_model::{AppModalState, Model, TimeoutType},
    tea_view::MAX_UI_WIDTH,
    ui_components::{
        modal_selector::ModalSelectorUpdate, Component, ModalSelector, ModalSelectorEvent,
        MsgModalSessionSelector, SelectableData, SelectorConfig, SelectorMode, TableColumn,
    },
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use opencode_sdk::models::File;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Borders, Cell, Widget},
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
    KeyInput(KeyEvent),
    FileSelected(File),
    Cancel,
}

/// File selector that wraps the generic ModalSelector
#[derive(Debug, Clone)]
pub struct FileSelector {
    pub modal: ModalSelector<FileData>,
    query: String,
    depth: u16,
    // Store both data sources separately
    file_status: Vec<File>,
    find_files_results: Vec<File>,
    // attachments
}

impl FileSelector {
    pub fn new() -> Self {
        let config = SelectorConfig {
            // title: "Files".to_string(),
            // footer: Some("↑↓/Tab navigate, Enter select, Esc cancel".to_string()),
            title: None,
            footer: None,
            max_width: Some(MAX_UI_WIDTH),
            max_height: Some(20),
            padding: 0,
            show_scrollbar: false,
            alternating_rows: true,
            borders: Borders::NONE,
            border_color: Color::Blue,
            selected_style: Style::default()
                .add_modifier(Modifier::REVERSED)
                .fg(Color::Blue),
            header_style: Style::default().fg(Color::Gray),
            row_style: Style::default().fg(Color::White),
            alt_row_style: None, // Some(Style::default().bg(Color::DarkGray)),
        };

        let columns = vec![
            TableColumn::new("Changes", Constraint::Length(10)),
            TableColumn::new("File Path", Constraint::Min(20)),
        ];

        Self {
            modal: ModalSelector::new(config, SelectorMode::Table { columns }),
            query: "".to_string(),
            depth: 0,
            file_status: Vec::new(),
            find_files_results: Vec::new(),
        }
    }

    pub fn set_files(&mut self, files: Vec<File>) {
        let file_data: Vec<FileData> = files.into_iter().map(FileData::from_file).collect();
        self.modal.set_items(file_data);
    }

    pub fn set_file_status(&mut self, files: Vec<File>) {
        self.file_status = files;
        self.update_combined_files();
    }

    pub fn set_find_files_results(&mut self, files: Vec<File>) {
        self.find_files_results = files;
        self.update_combined_files();
    }

    fn update_combined_files(&mut self) {
        use std::collections::HashMap;
        
        // Use HashMap to deduplicate by file path, with file_status taking precedence
        let mut combined_files: HashMap<String, File> = HashMap::new();
        
        // First add find files results
        for file in &self.find_files_results {
            combined_files.insert(file.path.clone(), file.clone());
        }
        
        // Then add file status, overwriting find files results for same paths
        for file in &self.file_status {
            combined_files.insert(file.path.clone(), file.clone());
        }
        
        // Convert to Vec and sort by path for consistent ordering
        let mut files: Vec<File> = combined_files.into_values().collect();
        files.sort_by(|a, b| a.path.cmp(&b.path));
        
        // Convert to FileData and set in the modal
        let file_data: Vec<FileData> = files.into_iter().map(FileData::from_file).collect();
        self.modal.set_items(file_data);
    }

    pub fn is_file_selector_input(key: KeyEvent) -> bool {
        !key.modifiers.contains(KeyModifiers::CONTROL)
            && !key.modifiers.contains(KeyModifiers::ALT)
            && matches!(key.code, KeyCode::Char(_) | KeyCode::Backspace)
    }

    pub fn clear(&mut self) {
        self.depth = 0;
        self.query = "".to_string();
        self.file_status.clear();
        self.find_files_results.clear();
        self.modal.set_items(Vec::new());
    }
}

fn model_select_file(file: File, model: &mut Model) {
    let current_text = model.text_input_area.content();
    let new_text = current_text.replace(&model.modal_file_selector.query, &file.path);
    model.text_input_area.set_content(&new_text);
    for _ in new_text.chars() {
        model
            .text_input_area
            .handle_input(KeyEvent::new(KeyCode::Right, KeyModifiers::NONE));
    }

    // TODO add attachment to here or to text input?
    // TODO how does deleting attachments work?
}

fn model_clear(model: &mut Model) {
    model.modal_file_selector.clear();
    model.state = AppModalState::None;
}

impl Component<Model, MsgModalFileSelector, ()> for FileSelector {
    fn update(msg: MsgModalFileSelector, state: &mut Model) -> CmdOrBatch<()> {
        let model = state;
        match msg {
            MsgModalFileSelector::Event(event) => {
                // Forward generic events to the file selector component
                match model.modal_file_selector.modal.handle_event(event) {
                    ModalSelectorUpdate::Hide => {
                        model_clear(model);
                    }
                    ModalSelectorUpdate::ItemSelected(file_data) => {
                        model_select_file(file_data.file, model);
                        model_clear(model);
                    }
                    _ => {}
                }
            }
            MsgModalFileSelector::FileSelected(file) => {
                model_select_file(file, model);
                model_clear(model);
            }
            MsgModalFileSelector::KeyInput(key) => {
                if FileSelector::is_file_selector_input(key) {
                    match key.code {
                        KeyCode::Backspace => {
                            if model.modal_file_selector.depth == 0 {
                                model_clear(model);
                            } else {
                                model.modal_file_selector.depth -= 1;
                                // Update query and set timeout for debounced search
                                if !model.modal_file_selector.query.is_empty() {
                                    model.modal_file_selector.query.pop();
                                    let query = model.modal_file_selector.query.clone();
                                    let timeout_type =
                                        TimeoutType::DebounceFindFiles(query.clone());
                                    model.set_timeout(timeout_type, 200); // 200ms debounce
                                }
                            }
                            model.text_input_area.handle_input(key);
                        }
                        KeyCode::Char(c) => {
                            if c == ' ' {
                                model_clear(model);
                            } else {
                                model.modal_file_selector.depth += 1;
                                model.modal_file_selector.query += &format!("{}", c);

                                // Set timeout for debounced file search
                                let query = model.modal_file_selector.query.clone();
                                let timeout_type = TimeoutType::DebounceFindFiles(query.clone());
                                model.set_timeout(timeout_type, 200); // 200ms debounce
                            }
                            model.text_input_area.handle_input(key);
                        }
                        _ => {}
                    }
                }
            }
            MsgModalFileSelector::Cancel => {
                model_clear(model);
            }
        };
        // File selector doesn't return Cmd, but the timeout system will trigger the search
        CmdOrBatch::Single(())
    }
}

impl Widget for &FileSelector {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.modal.render(area, buf);
    }
}
