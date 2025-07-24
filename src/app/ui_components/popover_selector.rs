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
    scroll_offset: usize,
    last_render_height: Option<usize>,
    current_session_index: Option<usize>,
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
            scroll_offset: 0,
            last_render_height: None,
            current_session_index: None,
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
            scroll_offset: 0,
            last_render_height: None,
            current_session_index: None,
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
                    self.update_scroll_position();
                }
                None
            }
            PopoverSelectorEvent::Down => {
                if !self.items.is_empty() {
                    self.selected_index = (self.selected_index + 1) % self.items.len();
                    self.update_scroll_position();
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
                self.scroll_offset = 0;
                self.current_session_index = None; // Reset when items change
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

    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    pub fn cache_render_height_for_terminal(&mut self, terminal_height: u16) {
        // Calculate what the popup height would be
        let popup_height = self
            .max_height
            .unwrap_or(self.items.len() as u16 + 4)
            .min(terminal_height);
        let content_height = popup_height.saturating_sub(2) as usize;
        self.last_render_height = Some(content_height);
    }

    pub fn set_current_session_index(&mut self, index: Option<usize>) {
        self.current_session_index = index;
    }

    pub fn current_session_index(&self) -> Option<usize> {
        self.current_session_index
    }

    pub fn set_max_dimensions(&mut self, max_width: Option<u16>, max_height: Option<u16>) {
        self.max_width = max_width;
        self.max_height = max_height;
    }

    pub fn update_scroll_position(&mut self) {
        let visible_height = self.last_render_height.unwrap_or_else(|| 3);
        if self.items.is_empty() || visible_height == 0 {
            return;
        }

        // Ensure selected item is visible within the scroll window
        if self.selected_index < self.scroll_offset {
            // Selected item is above the visible area, scroll up
            self.scroll_offset = self.selected_index;
        } else if self.selected_index >= self.scroll_offset + visible_height {
            // Selected item is below the visible area, scroll down
            self.scroll_offset = self.selected_index.saturating_sub(visible_height - 1);
        }

        // Ensure scroll offset doesn't exceed bounds
        let max_scroll = self.items.len().saturating_sub(visible_height);
        self.scroll_offset = self.scroll_offset.min(max_scroll);
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
        let content_height = area.height.saturating_sub(2) as usize;

        // Create title with scroll indicators
        let has_items_above = self.scroll_offset > 0;
        let visible_height = content_height.min(self.items.len());
        let has_items_below = self.scroll_offset + visible_height < self.items.len();

        let title = if has_items_above && has_items_below {
            format!("{} ↑↓", self.title)
        } else if has_items_above {
            format!("{} ↑", self.title)
        } else if has_items_below {
            format!("{} ↓", self.title)
        } else {
            self.title.clone()
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(Style::default().fg(Color::Blue));

        // Calculate visible range based on current scroll offset
        let start_index = self
            .scroll_offset
            .min(self.items.len().saturating_sub(visible_height));
        let end_index = start_index + visible_height;

        let items: Vec<ListItem> = self
            .items
            .iter()
            .enumerate()
            .skip(start_index)
            .take(end_index - start_index)
            .map(|(i, item)| {
                let (style, prefix) = if i == self.selected_index {
                    // Currently selected item (navigation cursor)
                    let style = Style::default().fg(Color::Black).bg(Color::White);
                    let prefix = if Some(i) == self.current_session_index {
                        ">*" // Selected AND current session
                    } else {
                        "> " // Just selected
                    };
                    (style, prefix)
                } else if Some(i) == self.current_session_index {
                    // Current active session (not selected)
                    let style = Style::default().fg(Color::Black).bg(Color::Green);
                    (style, " *")
                } else {
                    // Regular item
                    let style = Style::default().fg(Color::White);
                    (style, "  ")
                };

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
