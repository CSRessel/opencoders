use crate::app::{
    event_msg::{Cmd, CmdOrBatch},
    tea_model::{AppModalState, Model},
    ui_components::{
        Component, ModalSelector, ModalSelectorEvent, SelectableData, SelectorConfig, SelectorMode,
    },
};
use opencode_sdk::models::Session;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Cell, Widget},
};

/// Data wrapper for session selection
#[derive(Debug, Clone, PartialEq)]
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

/// Submessage enum for the session selector that wraps generic events
#[derive(Debug, Clone, PartialEq)]
pub enum MsgModalSessionSelector {
    Event(ModalSelectorEvent<SessionData>),
    SessionSelected(usize),
    CreateNew,
    Cancel,
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

impl Component<Model, MsgModalSessionSelector, Cmd> for SessionSelector {
    fn update(msg: MsgModalSessionSelector, state: &mut Model) -> CmdOrBatch<Cmd> {
        let model = state;
        match msg {
            MsgModalSessionSelector::Event(event) => {
                // Forward generic events to the session selector component
                // and handle any events it emits back
                if let Some(response_event) = model.modal_session_selector.modal.handle_event(event)
                {
                    // Handle response events
                    match response_event {
                        ModalSelectorEvent::Hide => {
                            model.state = AppModalState::None;
                        }
                        ModalSelectorEvent::ItemSelected(session_data) => {
                            // Convert session data back to index
                            if session_data.session.is_none() {
                                // "Create New" selected - index 0
                                if let Some(client) = model.client.clone() {
                                    if model.change_session(Some(0)) {
                                        return CmdOrBatch::Single(Cmd::AsyncSpawnSessionInit(
                                            client,
                                        ));
                                    }
                                }
                            } else {
                                // Find the session index
                                if let Some(session) = &session_data.session {
                                    let index =
                                        model.sessions.iter().position(|s| s.id == session.id);
                                    if let Some(client) = model.client.clone() {
                                        if model.change_session(index.map(|i| i + 1)) {
                                            // +1 for "Create New"
                                            return CmdOrBatch::Single(Cmd::AsyncSpawnSessionInit(
                                                client,
                                            ));
                                        }
                                    }
                                }
                            }
                            model.state = AppModalState::None;
                        }
                        _ => {}
                    }
                }
            }
            MsgModalSessionSelector::SessionSelected(index) => {
                if let Some(client) = model.client.clone() {
                    if model.change_session(Some(index)) {
                        return CmdOrBatch::Single(Cmd::AsyncSpawnSessionInit(client));
                    }
                }
                model.state = AppModalState::None;
            }
            MsgModalSessionSelector::CreateNew => {
                if let Some(client) = model.client.clone() {
                    if model.change_session(Some(0)) {
                        return CmdOrBatch::Single(Cmd::AsyncSpawnSessionInit(client));
                    }
                }
                model.state = AppModalState::None;
            }
            MsgModalSessionSelector::Cancel => {
                model.state = AppModalState::None;
            }
        };
        CmdOrBatch::Single(Cmd::None)
    }
}

impl Widget for &SessionSelector {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.modal.render(area, buf);
    }
}
