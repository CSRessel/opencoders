use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{
        Block, Borders, Cell, List, ListItem, Padding, Row, Scrollbar, ScrollbarOrientation,
        ScrollbarState, Table, TableState, Widget,
    },
};
use std::marker::PhantomData;

use crate::app::tea_view::MAX_UI_WIDTH;

/// Configuration for table columns
#[derive(Debug, Clone, PartialEq)]
pub struct TableColumn {
    pub header: String,
    pub constraint: Constraint,
    pub alignment: Option<ratatui::layout::Alignment>,
}

impl TableColumn {
    pub fn new<S: Into<String>>(header: S, constraint: Constraint) -> Self {
        Self {
            header: header.into(),
            constraint,
            alignment: None,
        }
    }

    pub fn with_alignment(mut self, alignment: ratatui::layout::Alignment) -> Self {
        self.alignment = Some(alignment);
        self
    }
}

/// Configuration for the modal selector appearance
#[derive(Debug, Clone)]
pub struct SelectorConfig {
    pub title: Option<String>,
    pub footer: Option<String>,
    pub max_width: Option<u16>,
    pub max_height: Option<u16>,
    pub padding: u16,
    pub show_scrollbar: bool,
    pub alternating_rows: bool,
    pub borders: Borders,
    pub border_color: Color,
    pub selected_style: Style,
    pub header_style: Style,
    pub row_style: Style,
    pub alt_row_style: Option<Style>,
}

impl Default for SelectorConfig {
    fn default() -> Self {
        Self {
            title: Some("Select".to_string()),
            footer: Some("↑↓ navigate, Enter select, Esc close".to_string()),
            max_width: Some(MAX_UI_WIDTH),
            max_height: Some(20),
            padding: 0,
            show_scrollbar: true,
            alternating_rows: false,
            borders: Borders::ALL,
            border_color: Color::Blue,
            selected_style: Style::default()
                // .add_modifier(Modifier::REVERSED)
                .fg(Color::Blue),
            header_style: Style::default().fg(Color::Gray),
            row_style: Style::default().fg(Color::White),
            alt_row_style: Some(Style::default().bg(Color::DarkGray)),
        }
    }
}

/// Trait for data that can be displayed in the modal selector
pub trait SelectableData: Clone {
    /// Convert the data item to table cells
    fn to_cells(&self) -> Vec<Cell>;

    /// Get a simple string representation (for list mode)
    fn to_string(&self) -> String;

    /// Optional: return styled spans for more complex formatting
    fn to_spans(&self) -> Option<Vec<Span>> {
        None
    }
}

/// Display mode for the selector
#[derive(Debug, Clone, PartialEq)]
pub enum SelectorMode {
    List,
    Table { columns: Vec<TableColumn> },
}

/// Generic events that can be handled by any modal selector
#[derive(Debug, Clone, PartialEq)]
pub enum ModalSelectorEvent<T>
where
    T: SelectableData + Clone,
{
    Show,
    Hide,
    KeyInput(KeyEvent),
    SetItems(Vec<T>),
    SetLoading(bool),
    SetError(Option<String>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ModalSelectorUpdate<T>
where
    T: SelectableData + Clone,
{
    Hide,
    ItemSelected(T),
    None,
}

/// Generic modal selector that can display different types of data
#[derive(Debug, Clone)]
pub struct ModalSelector<T>
where
    T: SelectableData + Clone,
{
    pub config: SelectorConfig,
    pub mode: SelectorMode,
    pub items: Vec<T>,
    pub state: TableState, // Used for both table and list selection
    pub scroll_state: ScrollbarState,
    pub is_visible: bool,
    pub loading: bool,
    pub error: Option<String>,
    _phantom: PhantomData<T>,
}

impl<T> ModalSelector<T>
where
    T: SelectableData + Clone,
{
    pub fn new(config: SelectorConfig, mode: SelectorMode) -> Self {
        let mut state = TableState::default();
        state.select(Some(0));

        Self {
            config,
            mode,
            items: Vec::new(),
            state,
            scroll_state: ScrollbarState::new(0),
            is_visible: false,
            loading: false,
            error: None,
            _phantom: PhantomData,
        }
    }

    pub fn list(title: &str) -> Self {
        Self::new(
            SelectorConfig {
                title: Some(title.to_string()),
                ..Default::default()
            },
            SelectorMode::List,
        )
    }

    pub fn table(title: &str, columns: Vec<TableColumn>) -> Self {
        Self::new(
            SelectorConfig {
                title: Some(title.to_string()),
                ..Default::default()
            },
            SelectorMode::Table { columns },
        )
    }

    // State management methods
    pub fn show(&mut self) {
        self.is_visible = true;
    }

    pub fn hide(&mut self) {
        self.is_visible = false;
    }

    pub fn is_visible(&self) -> bool {
        self.is_visible
    }

    pub fn set_loading(&mut self, loading: bool) {
        self.loading = loading;
        if loading {
            self.error = None;
        }
    }

    pub fn set_error(&mut self, error: Option<String>) {
        self.error = error;
        self.loading = false;
    }

    pub fn set_items(&mut self, items: Vec<T>) {
        self.items = items;
        self.scroll_state = ScrollbarState::new(self.items.len());
        self.state
            .select(if self.items.is_empty() { None } else { Some(0) });
        self.loading = false;
        self.error = None;
    }

    // Navigation methods
    pub fn navigate_up(&mut self) {
        if self.items.is_empty() {
            return;
        }

        let current = self.state.selected().unwrap_or(0);
        let new_index = if current == 0 {
            self.items.len() - 1
        } else {
            current - 1
        };
        self.state.select(Some(new_index));
    }

    pub fn navigate_down(&mut self) {
        if self.items.is_empty() {
            return;
        }

        let current = self.state.selected().unwrap_or(0);
        let new_index = if current >= self.items.len() - 1 {
            0
        } else {
            current + 1
        };
        self.state.select(Some(new_index));
    }

    pub fn selected_index(&self) -> Option<usize> {
        self.state.selected()
    }

    pub fn selected_item(&self) -> Option<&T> {
        self.selected_index().and_then(|i| self.items.get(i))
    }

    pub fn items(&self) -> &[T] {
        &self.items
    }

    // Generic event handling
    pub fn handle_event(&mut self, event: ModalSelectorEvent<T>) -> ModalSelectorUpdate<T> {
        match event {
            ModalSelectorEvent::Show => {
                self.show();
            }
            ModalSelectorEvent::Hide => {
                self.hide();
            }
            ModalSelectorEvent::SetItems(items) => {
                self.set_items(items);
            }
            ModalSelectorEvent::SetLoading(loading) => {
                self.set_loading(loading);
            }
            ModalSelectorEvent::SetError(error) => {
                self.set_error(error);
            }
            ModalSelectorEvent::KeyInput(key) => return self.handle_key_input(key),
        };
        ModalSelectorUpdate::None
    }

    pub fn is_modal_selector_input(key_code: KeyCode) -> bool {
        matches!(
            key_code,
            KeyCode::Esc | KeyCode::Up | KeyCode::Down | KeyCode::Tab | KeyCode::Enter
        )
    }

    fn handle_key_input(&mut self, key: KeyEvent) -> ModalSelectorUpdate<T> {
        match key.code {
            KeyCode::Esc => ModalSelectorUpdate::Hide,
            KeyCode::BackTab | KeyCode::Up => {
                self.navigate_up();
                ModalSelectorUpdate::None
            }
            KeyCode::Tab | KeyCode::Down => {
                self.navigate_down();
                ModalSelectorUpdate::None
            }
            KeyCode::Enter => {
                if let Some(item) = self.selected_item() {
                    ModalSelectorUpdate::ItemSelected(item.clone())
                } else {
                    ModalSelectorUpdate::None
                }
            }
            _ => ModalSelectorUpdate::None,
        }
    }

    // Rendering methods
    fn render_loading(&self, area: Rect, buf: &mut Buffer) {
        let mut block = Block::default()
            .padding(Padding::uniform(self.config.padding))
            .borders(self.config.borders)
            .border_style(Style::default().fg(self.config.border_color));
        if let Some(title) = &self.config.title {
            block = block.title_top(title.clone())
        }

        let loading_text = Text::from("Loading...");
        let paragraph = ratatui::widgets::Paragraph::new(loading_text)
            .style(Style::default().fg(Color::Yellow))
            .block(block);

        paragraph.render(area, buf);
    }

    fn render_error(&self, area: Rect, buf: &mut Buffer, error: &str) {
        let mut block = Block::default()
            .padding(Padding::uniform(self.config.padding))
            .borders(self.config.borders)
            .border_style(Style::default().fg(Color::Red));
        if let Some(title) = &self.config.title {
            block = block.title_top(title.clone())
        }

        let error_text = Text::from(format!("Error: {}", error));
        let paragraph = ratatui::widgets::Paragraph::new(error_text)
            .style(Style::default().fg(Color::Red))
            .block(block);

        paragraph.render(area, buf);
    }

    fn render_list(&self, area: Rect, buf: &mut Buffer) {
        let mut block = Block::default()
            .padding(Padding::uniform(self.config.padding))
            .borders(self.config.borders)
            .border_style(Style::default().fg(self.config.border_color));
        if let Some(title) = &self.config.title {
            block = block.title_top(title.clone())
        }
        if let Some(footer) = &self.config.footer {
            block = block.title_bottom(footer.clone())
        }

        if self.items.is_empty() {
            let empty_text = Text::from("No items found");
            let paragraph = ratatui::widgets::Paragraph::new(empty_text)
                .style(self.config.row_style)
                .block(block);
            paragraph.render(area, buf);
            return;
        }

        let items: Vec<ListItem> = self
            .items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let style = if Some(i) == self.selected_index() {
                    self.config.selected_style
                } else {
                    self.config.row_style
                };

                let content = if let Some(spans) = item.to_spans() {
                    Line::from(spans)
                } else {
                    Line::from(item.to_string())
                };

                ListItem::new(content).style(style)
            })
            .collect();

        let list = List::new(items).block(block);
        list.render(area, buf);
    }

    fn render_table(&self, area: Rect, buf: &mut Buffer, columns: &[TableColumn]) {
        let mut block = Block::default()
            .padding(Padding::uniform(self.config.padding))
            .borders(self.config.borders)
            .border_style(Style::default().fg(self.config.border_color));
        if let Some(title) = &self.config.title {
            block = block.title_top(title.clone())
        }
        if let Some(footer) = &self.config.footer {
            block = block.title_bottom(footer.clone())
        }

        if self.items.is_empty() {
            let empty_table = Table::new(
                [Row::new([Cell::from("No matching items")])],
                [Constraint::Percentage(100)],
            )
            .block(block);
            empty_table.render(area, buf);
            return;
        }

        // Create header
        let header = Row::new(
            columns
                .iter()
                .map(|col| Cell::from(col.header.clone()))
                .collect::<Vec<_>>(),
        )
        .style(self.config.header_style)
        .height(1);

        // Create rows
        let rows = self.items.iter().enumerate().map(|(i, item)| {
            let style = if Some(i) == self.selected_index() {
                self.config.selected_style
            } else if self.config.alternating_rows && i % 2 == 1 {
                self.config.alt_row_style.unwrap_or(self.config.row_style)
            } else {
                self.config.row_style
            };

            Row::new(item.to_cells()).style(style).height(1)
        });

        // Extract constraints from columns
        let constraints: Vec<Constraint> = columns.iter().map(|col| col.constraint).collect();

        let table = Table::new(rows, constraints)
            .header(header)
            .block(block)
            .row_highlight_style(self.config.selected_style);

        // Need to render with mutable state
        let mut mutable_state = self.state.clone();
        ratatui::widgets::StatefulWidget::render(table, area, buf, &mut mutable_state);

        // Render scrollbar if enabled
        if self.config.show_scrollbar
            && self.items.len()
                > (area
                    .height
                    .saturating_sub(2)
                    .saturating_sub(self.config.padding * 2)) as usize
        {
            let scrollbar_area = Rect {
                x: area.x + area.width - 1,
                y: area.y + 1,
                width: 1,
                height: area.height - 2,
            };

            let mut scroll_state = self.scroll_state.clone();
            let scrollbar = Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .begin_symbol(None)
                .end_symbol(None);
            ratatui::widgets::StatefulWidget::render(
                scrollbar,
                scrollbar_area,
                buf,
                &mut scroll_state,
            );
        }
    }

    fn calculate_popup_area(&self, area: Rect) -> Rect {
        let popup_width = self.config.max_width.unwrap_or(area.width).min(area.width);

        let popup_height = match &self.mode {
            SelectorMode::List => (self.items.len() as u16)
                .saturating_add(2)
                .saturating_add(self.config.padding * 2),
            SelectorMode::Table { .. } => (self.items.len() as u16)
                .saturating_add(2)
                .saturating_add(self.config.padding * 2),
        }
        .min(self.config.max_height.unwrap_or(area.height));

        // Center the popup
        let popup_x = (area.width.saturating_sub(popup_width)) / 2;
        let popup_y = (area.height.saturating_sub(popup_height)) / 2;

        Rect {
            x: area.x + popup_x,
            y: area.y + popup_y,
            width: popup_width,
            height: popup_height,
        }
    }
}

impl<T> Widget for &ModalSelector<T>
where
    T: SelectableData + Clone,
{
    fn render(self, area: Rect, buf: &mut Buffer) {
        if !self.is_visible {
            return;
        }

        let popup_area = self.calculate_popup_area(area);

        // Clear the popup area (overlay effect)
        for y in popup_area.y..popup_area.y + popup_area.height {
            for x in popup_area.x..popup_area.x + popup_area.width {
                if x < buf.area.width && y < buf.area.height {
                    buf[(x, y)].reset();
                }
            }
        }

        // Render content based on state
        if self.loading {
            self.render_loading(popup_area, buf);
        } else if let Some(error) = &self.error {
            self.render_error(popup_area, buf, error);
        } else {
            match &self.mode {
                SelectorMode::List => self.render_list(popup_area, buf),
                SelectorMode::Table { columns } => self.render_table(popup_area, buf, columns),
            }
        }
    }
}

