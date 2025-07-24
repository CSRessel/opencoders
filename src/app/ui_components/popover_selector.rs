use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Widget},
};

#[derive(Debug, Clone, PartialEq)]
pub struct PopoverSelector {
    title: String,
    items: Vec<String>,
    selected_index: usize,
    is_visible: bool,
    loading: bool,
    error: Option<String>,
    max_height: Option<u16>,
    max_width: Option<u16>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PopoverSelectorEvent {
    Up,
    Down,
    Select,
    Cancel,
    SetItems(Vec<String>),
    SetLoading(bool),
    SetError(Option<String>),
    Show,
    Hide,
}

impl PopoverSelector {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            items: Vec::new(),
            selected_index: 0,
            is_visible: false,
            loading: false,
            error: None,
            max_height: None,
            max_width: None,
        }
    }

    pub fn with_items(title: &str, items: Vec<String>) -> Self {
        Self {
            title: title.to_string(),
            items,
            selected_index: 0,
            is_visible: false,
            loading: false,
            error: None,
            max_height: None,
            max_width: None,
        }
    }

    pub fn handle_event(&mut self, event: PopoverSelectorEvent) -> Option<usize> {
        match event {
            PopoverSelectorEvent::Up => {
                if !self.items.is_empty() {
                    self.selected_index = if self.selected_index == 0 {
                        self.items.len() - 1
                    } else {
                        self.selected_index - 1
                    };
                }
                None
            }
            PopoverSelectorEvent::Down => {
                if !self.items.is_empty() {
                    self.selected_index = (self.selected_index + 1) % self.items.len();
                }
                None
            }
            PopoverSelectorEvent::Select => {
                if !self.items.is_empty() && self.is_visible {
                    Some(self.selected_index)
                } else {
                    None
                }
            }
            PopoverSelectorEvent::Cancel => {
                self.is_visible = false;
                None
            }
            PopoverSelectorEvent::SetItems(items) => {
                self.items = items;
                self.selected_index = 0;
                self.loading = false;
                self.error = None;
                None
            }
            PopoverSelectorEvent::SetLoading(loading) => {
                self.loading = loading;
                if loading {
                    self.error = None;
                }
                None
            }
            PopoverSelectorEvent::SetError(error) => {
                self.error = error;
                self.loading = false;
                None
            }
            PopoverSelectorEvent::Show => {
                self.is_visible = true;
                None
            }
            PopoverSelectorEvent::Hide => {
                self.is_visible = false;
                None
            }
        }
    }

    pub fn is_visible(&self) -> bool {
        self.is_visible
    }

    pub fn selected_index(&self) -> usize {
        self.selected_index
    }

    pub fn items(&self) -> &[String] {
        &self.items
    }

    pub fn is_loading(&self) -> bool {
        self.loading
    }

    pub fn error(&self) -> Option<&String> {
        self.error.as_ref()
    }

    pub fn set_max_dimensions(&mut self, max_width: Option<u16>, max_height: Option<u16>) {
        self.max_width = max_width;
        self.max_height = max_height;
    }

    fn render_loading(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(self.title.clone())
            .border_style(Style::default().fg(Color::Blue));

        let loading_text = "Loading...";
        let paragraph = Paragraph::new(Line::from(Span::styled(
            loading_text,
            Style::default().fg(Color::Yellow),
        )))
        .block(block);

        paragraph.render(area, buf);
    }

    fn render_error(&self, area: Rect, buf: &mut Buffer, error: &str) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(self.title.clone())
            .border_style(Style::default().fg(Color::Red));

        let error_text = format!("Error: {}", error);
        let paragraph = Paragraph::new(Line::from(Span::styled(
            error_text,
            Style::default().fg(Color::Red),
        )))
        .block(block);

        paragraph.render(area, buf);
    }

    fn render_items(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(self.title.clone())
            .border_style(Style::default().fg(Color::Blue));

        let items: Vec<ListItem> = self
            .items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let style = if i == self.selected_index {
                    Style::default().fg(Color::Black).bg(Color::White)
                } else {
                    Style::default().fg(Color::White)
                };

                let prefix = if i == self.selected_index { "> " } else { "  " };
                ListItem::new(Line::from(Span::styled(
                    format!("{}{}", prefix, item),
                    style,
                )))
            })
            .collect();

        let list = List::new(items).block(block);
        list.render(area, buf);
    }
}

impl Widget for &PopoverSelector {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if !self.is_visible {
            return;
        }

        // Calculate popup dimensions
        let popup_width = self
            .max_width
            .unwrap_or(60)
            .min(area.width.saturating_sub(4));
        let popup_height = self
            .max_height
            .unwrap_or(self.items.len() as u16 + 4)
            .min(area.height);

        // Center the popup
        let popup_x = (area.width.saturating_sub(popup_width)) / 2;
        let popup_y = (area.height.saturating_sub(popup_height)) / 2;

        let popup_area = Rect {
            x: area.x + popup_x,
            y: area.y + popup_y,
            width: popup_width,
            height: popup_height,
        };

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
            self.render_items(popup_area, buf);
        }
    }
}

impl Default for PopoverSelector {
    fn default() -> Self {
        Self::new("Select")
    }
}

