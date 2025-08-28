use crate::app::ui_components::{ModalSelector, SelectableData, SelectorConfig, SelectorMode};
use opencode_sdk::models::Session;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Cell, Widget},
};

/// Data wrapper for session selection
#[derive(Debug, Clone)]
pub struct SessionData {
    pub session: Option<Session>,
    pub display_text: String,
    pub is_current: bool,
}

impl SessionData {
    pub fn new_session() -> Self {
        Self {
            session: None,
            display_text: "Create New Session".to_string(),
            is_current: false,
        }
    }

    pub fn from_session(session: &Session, is_current: bool) -> Self {
        Self {
            display_text: session.title.clone(),
            session: Some(session.clone()),
            is_current,
        }
    }
}

impl SelectableData for SessionData {
    fn to_cells(&self) -> Vec<Cell> {
        vec![Cell::from(self.to_string())]
    }

    fn to_string(&self) -> String {
        self.display_text.clone()
    }

    fn to_spans(&self) -> Option<Vec<Span>> {
        let prefix = if self.is_current { "* " } else { "  " };

        Some(vec![
            Span::styled(
                prefix,
                if self.is_current {
                    Style::default().fg(Color::Blue)
                } else {
                    Style::default()
                },
            ),
            Span::raw(&self.display_text),
        ])
    }
}

/// Events that can be sent to the SessionSelector
#[derive(Debug, Clone, PartialEq)]
pub enum SessionEvent {
    Up,
    Down,
    Select,
    Cancel,
    SetItems(Vec<Session>, Option<usize>),
    SetLoading(bool),
    SetError(Option<String>),
    Show,
    Hide,
}

/// Session selector that wraps the generic ModalSelector
#[derive(Debug, Clone)]
pub struct SessionSelector {
    pub modal: ModalSelector<SessionData>,
    sessions: Vec<Session>,
    current_session_index: Option<usize>,
}

impl SessionSelector {
    pub fn new(title: &str) -> Self {
        let config = SelectorConfig {
            title: title.to_string(),
            footer: Some("↑↓ navigate, Enter select, Esc cancel".to_string()),
            max_width: Some(60),
            max_height: Some(15),
            show_scrollbar: false,
            alternating_rows: false,
            border_color: Color::Blue,
            selected_style: Style::default()
                .add_modifier(Modifier::REVERSED)
                .fg(Color::White)
                .bg(Color::Blue),
            header_style: Style::default().fg(Color::Yellow),
            row_style: Style::default().fg(Color::White),
            alt_row_style: None,
        };

        Self {
            modal: ModalSelector::new(config, SelectorMode::List),
            sessions: Vec::new(),
            current_session_index: None,
        }
    }

    // Event handling similar to the original PopoverSelector
    pub fn handle_event(&mut self, event: SessionEvent) -> Option<usize> {
        match event {
            SessionEvent::Up => {
                self.modal.navigate_up();
                None
            }
            SessionEvent::Down => {
                self.modal.navigate_down();
                None
            }
            SessionEvent::Select => {
                if self.modal.is_visible() {
                    self.modal.selected_index()
                } else {
                    None
                }
            }
            SessionEvent::Cancel => {
                self.modal.hide();
                None
            }
            SessionEvent::SetItems(sessions, current_index) => {
                // Convert string items to SessionData
                let mut session_data = vec![SessionData::new_session()]; // Always include "Create New"

                tracing::debug!(
                    "handling {} sessions (current {:?})!!!",
                    sessions.len(),
                    current_index
                );

                for (i, session) in sessions.iter().enumerate() {
                    let is_current = current_index == Some(i);
                    self.current_session_index = Some(i + 1); // +1 because of "Create New"
                    session_data.push(SessionData::from_session(session, is_current));
                }

                self.modal.set_items(session_data);
                self.modal.set_loading(false);
                self.modal.set_error(None);
                None
            }
            SessionEvent::SetLoading(loading) => {
                self.modal.set_loading(loading);
                None
            }
            SessionEvent::SetError(error) => {
                self.modal.set_error(error);
                None
            }
            SessionEvent::Show => {
                self.modal.show();
                None
            }
            SessionEvent::Hide => {
                self.modal.hide();
                None
            }
        }
    }

    // Compatibility methods from original PopoverSelector
    pub fn is_visible(&self) -> bool {
        self.modal.is_visible()
    }

    pub fn selected_index(&self) -> usize {
        self.modal.selected_index().unwrap_or(0)
    }

    pub fn items(&self) -> Vec<String> {
        self.modal
            .items()
            .iter()
            .map(|item| item.to_string())
            .collect()
    }

    pub fn is_loading(&self) -> bool {
        self.modal.loading
    }

    pub fn error(&self) -> Option<&String> {
        self.modal.error.as_ref()
    }

    pub fn scroll_offset(&self) -> usize {
        // Not directly accessible from ModalSelector, return 0 for compatibility
        0
    }

    pub fn set_current_session_index(&mut self, index: Option<usize>) {
        self.current_session_index = index;
        // Need to update the display of current sessions
        // self.update_session_display();
    }

    pub fn current_session_index(&self) -> Option<usize> {
        self.current_session_index
    }

    pub fn set_max_dimensions(&mut self, max_width: Option<u16>, max_height: Option<u16>) {
        self.modal.config.max_width = max_width;
        self.modal.config.max_height = max_height;
    }

    pub fn update_scroll_position(&mut self) {
        // The generic ModalSelector handles scrolling automatically
    }

    // // Additional methods for session management
    // pub fn set_sessions(&mut self, sessions: Vec<Session>) {
    //     self.sessions = sessions;
    //     self.update_session_display();
    // }
    // fn update_session_display(&mut self) {
    //     let mut session_data = vec![SessionData::new_session()];
    //     for (i, session) in self.sessions.iter().enumerate() {
    //         let is_current = self.current_session_index == Some(i + 1); // +1 because of "Create New"
    //         session_data.push(SessionData::from_session(session, is_current));
    //     }
    //     self.modal.set_items(session_data);
    // }
}

impl Widget for &SessionSelector {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.modal.render(area, buf);
    }
}

