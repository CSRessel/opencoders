use crate::{
    app::ui_components::{MessageLog, TextInput},
    sdk::{OpenCodeClient, OpenCodeError},
};
use opencode_sdk::models::Session;

#[derive(Debug, Clone, PartialEq)]
pub struct Model {
    pub init: ModelInit,
    // App state
    pub state: AppState,
    pub input_history: Vec<String>,
    pub last_input: Option<String>,
    pub printed_to_stdout_count: usize,
    // Stateful components:
    pub message_log: MessageLog,
    pub text_input: TextInput,
    // Client and session state
    pub client: Option<OpenCodeClient>,
    pub session: Option<Session>,
    pub connection_status: ConnectionStatus,
}

mod model_init {
    #[derive(Debug, Clone, PartialEq)]
    pub struct ModelInit {
        // Immutable initialization properties
        // that can't be changed without restarting the terminal
        init_height: u16,
        init_inline_mode: bool,
    }

    impl ModelInit {
        pub fn height(&self) -> u16 {
            self.init_height
        }

        pub fn inline_mode(&self) -> bool {
            self.init_inline_mode
        }
        pub fn new(height: u16, inline_mode: bool) -> ModelInit {
            ModelInit {
                init_height: height,
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
    ConnectionError(String),
    Quit,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
    InitializingSession,
    SessionReady,
    Error(String),
}

impl Model {
    pub fn new() -> Self {
        let mut text_input = TextInput::new();
        text_input.set_focus(true);

        let message_log = MessageLog::new();

        Model {
            init: ModelInit::new(5, false),
            state: AppState::Welcome,
            input_history: Vec::new(),
            last_input: None,
            printed_to_stdout_count: 0,
            message_log,
            text_input,
            client: None,
            session: None,
            connection_status: ConnectionStatus::Disconnected,
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
        self.connection_status = ConnectionStatus::InitializingSession;
        self.state = AppState::InitializingSession;
    }

    pub fn transition_to_error(&mut self, error_msg: String) {
        self.connection_status = ConnectionStatus::Error(error_msg.clone());
        self.state = AppState::ConnectionError(error_msg);
    }

    pub fn transition_to_session_ready(&mut self, session: Session) {
        self.text_input.set_session_id(Some(session.id.clone()));
        self.session = Some(session);
        self.connection_status = ConnectionStatus::SessionReady;
        self.state = AppState::TextEntry;
        self.message_log.scroll_to_bottom();
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
                ConnectionStatus::Connected | ConnectionStatus::SessionReady
            )
    }

    pub fn is_session_ready(&self) -> bool {
        self.client.is_some()
            && self.session.is_some()
            && matches!(self.connection_status, ConnectionStatus::SessionReady)
    }

    pub fn client(&self) -> Option<&OpenCodeClient> {
        self.client.as_ref()
    }

    pub fn session(&self) -> Option<&Session> {
        self.session.as_ref()
    }
}
