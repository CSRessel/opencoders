use crate::{
    app::ui_components::{MessageLog, PopoverSelector, PopoverSelectorEvent, TextInput},
    sdk::{OpenCodeClient, OpenCodeError},
};
use opencode_sdk::models::Session;
use std::time::SystemTime;

#[derive(Debug, Clone, PartialEq)]
pub struct PendingSessionInfo {
    pub temp_id: String,
    pub created_at: SystemTime,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SessionState {
    None,
    Pending(PendingSessionInfo),
    Creating(PendingSessionInfo),
    Ready(Session),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Model {
    pub init: ModelInit,
    pub height: u16,
    pub ui_is_rounded: bool,
    // App state
    pub state: AppState,
    pub input_history: Vec<String>,
    pub last_input: Option<String>,
    pub printed_to_stdout_count: usize,
    // Stateful components:
    pub message_log: MessageLog,
    pub text_input: TextInput,
    pub session_selector: PopoverSelector,
    // Client and session state
    pub client: Option<OpenCodeClient>,
    pub session_state: SessionState,
    pub sessions: Vec<Session>,
    pub connection_status: ConnectionStatus,
    pub pending_first_message: Option<String>,
}

mod model_init {
    #[derive(Debug, Clone, PartialEq)]
    pub struct ModelInit {
        // Immutable initialization properties
        // that can't be changed without restarting the terminal
        init_inline_mode: bool,
    }

    impl ModelInit {
        pub fn inline_mode(&self) -> bool {
            self.init_inline_mode
        }

        pub fn new(inline_mode: bool) -> ModelInit {
            ModelInit {
                init_inline_mode: inline_mode,
            }
        }
    }
}

pub use model_init::ModelInit;

#[derive(Debug, Clone, PartialEq)]
pub enum AppState {
    Welcome,
    ConnectingToServer,
    InitializingSession,
    TextEntry,
    SelectSession,
    ConnectionError(String),
    Quit,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
    ClientReady,
    InitializingSession,
    SessionReady,
    Error(String),
}

const DEFAULT_HEIGHT: u16 = 12;
const DEFAULT_UI_IS_ROUNDED: bool = true;

impl Model {
    pub fn new() -> Self {
        let mut text_input = TextInput::new();
        text_input.set_focus(true);

        let message_log = MessageLog::new();
        let session_selector = PopoverSelector::new("Select Session");

        Model {
            init: ModelInit::new(true),
            height: DEFAULT_HEIGHT,
            ui_is_rounded: DEFAULT_UI_IS_ROUNDED,
            state: AppState::ConnectingToServer,
            input_history: Vec::new(),
            last_input: None,
            printed_to_stdout_count: 0,
            message_log,
            text_input,
            session_selector,
            client: None,
            session_state: SessionState::None,
            sessions: Vec::new(),
            connection_status: ConnectionStatus::Connecting,
            pending_first_message: None,
        }
    }

    // Message outputs
    pub fn needs_manual_output(&self) -> bool {
        return self.init.inline_mode() & (self.messages_needing_stdout_print().len() > 0);
    }

    pub fn messages_needing_stdout_print(&self) -> &[String] {
        // All messages that haven't been printed to stdout yet
        let printed_count = self.printed_to_stdout_count;
        if printed_count < self.input_history.len() {
            &self.input_history[printed_count..]
        } else {
            &[]
        }
    }

    // State transition helpers
    pub fn transition_to_connecting(&mut self) {
        self.state = AppState::ConnectingToServer;
        self.connection_status = ConnectionStatus::Connecting;
    }

    pub fn transition_to_connected(&mut self) {
        self.connection_status = ConnectionStatus::ClientReady;
        self.state = AppState::Welcome;
    }

    pub fn transition_to_error(&mut self, error_msg: String) {
        self.connection_status = ConnectionStatus::Error(error_msg.clone());
        self.state = AppState::ConnectionError(error_msg);
    }

    pub fn mark_messages_printed_to_stdout(&mut self, count: usize) {
        self.printed_to_stdout_count += count;
    }

    // Input management
    pub fn clear_input_state(&mut self) {
        self.text_input.clear();
        self.last_input = None;
        self.input_history.clear();
        self.printed_to_stdout_count = 0;
    }

    // Convenience accessors
    pub fn client_base_url(&self) -> &str {
        self.client().map(|c| c.base_url()).unwrap_or("unknown")
    }

    pub fn is_client_ready(&self) -> bool {
        self.client.is_some()
            && matches!(
                self.connection_status,
                ConnectionStatus::Connected
                    | ConnectionStatus::ClientReady
                    | ConnectionStatus::SessionReady
            )
    }

    pub fn is_session_ready(&self) -> bool {
        self.client.is_some()
            && matches!(self.session_state, SessionState::Ready(_))
            && matches!(self.connection_status, ConnectionStatus::SessionReady)
    }

    pub fn client(&self) -> Option<&OpenCodeClient> {
        self.client.as_ref()
    }

    pub fn session(&self) -> Option<&Session> {
        match &self.session_state {
            SessionState::Ready(session) => Some(session),
            _ => None,
        }
    }

    pub fn change_session_by_index(&mut self, index: Option<usize>) {
        self.message_log.set_messages(vec![]);
        self.text_input.set_session_id(None); // This will be handled in the Cmd callback
        self.session_selector.set_current_session_index(index);
        self.session_selector
            .handle_event(PopoverSelectorEvent::Hide);
    }

    pub fn change_session(&mut self, index: Option<usize>) -> bool {
        match index {
            // Handle selection
            Some(0) => {
                self.change_session_by_index(None);
                self.state = AppState::TextEntry;

                // Create pending session info
                let pending_info = PendingSessionInfo {
                    temp_id: uuid::Uuid::new_v4().to_string(),
                    created_at: SystemTime::now(),
                };
                self.session_state = SessionState::Pending(pending_info);
            }
            Some(requested_session_index) => {
                // Use existing session (requested_session_index - 1 in sessions list)
                let session_index = requested_session_index - 1;
                if session_index < self.sessions.len() {
                    self.change_session_by_index(Some(requested_session_index));
                    self.state = AppState::InitializingSession;
                    self.connection_status = ConnectionStatus::InitializingSession;

                    return true;
                }
            }
            None => {}
        };
        false
    }

    pub fn current_session_id(&self) -> Option<String> {
        match &self.session_selector.current_session_index() {
            None => None,
            Some(0) => None,
            Some(n) => self.sessions.get(n - 1).map(|session| session.id.clone()),
        }
    }

    pub fn has_pending_or_creating_session(&self) -> bool {
        matches!(
            self.session_state,
            SessionState::Pending(_) | SessionState::Creating(_)
        )
    }

    pub fn can_accept_input(&self) -> bool {
        matches!(
            self.session_state,
            SessionState::Pending(_) | SessionState::Creating(_) | SessionState::Ready(_)
        )
    }
}
