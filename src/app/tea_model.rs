use crate::{
    app::{
        message_state::MessageState,
        ui_components::{MessageLog, PopoverSelector, PopoverSelectorEvent, TextInput},
    },
    sdk::{extensions::events::EventStreamHandle, OpenCodeClient, OpenCodeError},
};
use opencode_sdk::models::{Mode, Session};
use std::time::SystemTime;

#[derive(Debug, Clone, PartialEq)]
pub enum RepeatShortcutKey {
    CtrlC,
    CtrlD,
    Esc,
    Leader,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RepeatShortcutTimeout {
    pub key: RepeatShortcutKey,
    pub started_at: SystemTime,
}

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
pub enum EventStreamState {
    Disconnected,
    Connecting,
    Connected(EventStreamHandle),
    Reconnecting { attempt: u32, last_error: String },
    Failed(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Model {
    pub init: ModelInit,
    pub config: UserConfig,
    // App state
    pub state: AppState,
    pub input_history: Vec<String>,
    pub last_input: Option<String>,
    pub printed_to_stdout_count: usize,
    pub sdk_mode: String,
    pub sdk_provider: String,
    pub sdk_model: String,
    // Stateful components:
    pub message_log: MessageLog,
    pub text_input: TextInput,
    pub session_selector: PopoverSelector,
    // Client and session state
    pub client: Option<OpenCodeClient>,
    pub session_state: SessionState,
    pub sessions: Vec<Session>,
    pub modes: Vec<Mode>,
    pub mode_state: Option<usize>,
    pub connection_status: ConnectionStatus,
    pub pending_first_message: Option<String>,
    // Message state and event streaming
    pub message_state: MessageState,
    pub event_stream_state: EventStreamState,
    // Unified repeat shortcut timeout system
    pub repeat_shortcut_timeout: Option<RepeatShortcutTimeout>,
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

#[derive(Debug, Clone, PartialEq)]
pub struct UserConfig {
    pub ui_block_is_rounded: bool,
    pub ui_block_is_bordered: bool,
    pub ui_block_padding: u16,
    pub ui_status_is_bottom: bool,
    pub ui_status_use_labels: bool,
    pub height: u16,
    pub keys_shortcut_timeout_ms: u16,
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

impl Model {
    pub fn new() -> Self {
        let mut text_input = TextInput::new();
        text_input.set_focus(true);

        let message_log = MessageLog::new();
        let session_selector = PopoverSelector::new("Select Session");

        Model {
            init: ModelInit::new(true),
            config: UserConfig {
                ui_block_is_rounded: true,
                ui_block_is_bordered: true,
                ui_block_padding: 0,
                ui_status_is_bottom: true,
                ui_status_use_labels: true,
                height: DEFAULT_HEIGHT,
                keys_shortcut_timeout_ms: 1000,
            },
            state: AppState::ConnectingToServer,
            input_history: Vec::new(),
            last_input: None,
            printed_to_stdout_count: 0,
            sdk_mode: "chat".to_string(),
            sdk_provider: "anthropic".to_string(),
            sdk_model: "claude-sonnet-4-20250514".to_string(),
            message_log,
            text_input,
            session_selector,
            client: None,
            session_state: SessionState::None,
            sessions: Vec::new(),
            modes: Vec::new(),
            mode_state: None,
            connection_status: ConnectionStatus::Connecting,
            pending_first_message: None,
            message_state: MessageState::new(),
            event_stream_state: EventStreamState::Disconnected,
            repeat_shortcut_timeout: None,
        }
    }

    // Message outputs
    pub fn needs_manual_output(&self) -> bool {
        return self.init.inline_mode() & self.message_state.has_messages_needing_stdout_print();
    }

    pub fn messages_needing_stdout_print(&self) -> Vec<String> {
        // All messages that haven't been printed to stdout yet from message state
        self.message_state.get_messages_needing_stdout_print()
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
        self.message_state.mark_messages_printed_to_stdout(count);
        // Keep the old counter for backward compatibility with input_history
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

    // Unified repeat shortcut timeout management
    pub fn set_repeat_shortcut_timeout(&mut self, key: RepeatShortcutKey) {
        self.repeat_shortcut_timeout = Some(RepeatShortcutTimeout {
            key,
            started_at: SystemTime::now(),
        });
    }

    pub fn clear_repeat_shortcut_timeout(&mut self) {
        self.repeat_shortcut_timeout = None;
    }

    pub fn is_repeat_shortcut_timeout_active(&self, key: RepeatShortcutKey) -> bool {
        if let Some(timeout) = &self.repeat_shortcut_timeout {
            if timeout.key == key {
                if let Ok(elapsed) = timeout.started_at.elapsed() {
                    return elapsed.as_secs() < 1;
                }
            }
        }
        false
    }

    pub fn has_active_timeout(&self) -> bool {
        if let Some(timeout) = &self.repeat_shortcut_timeout {
            if let Ok(elapsed) = timeout.started_at.elapsed() {
                return elapsed.as_secs() < 1;
            }
        }
        false
    }

    pub fn expire_timeout_if_needed(&mut self) -> bool {
        if let Some(timeout) = &self.repeat_shortcut_timeout {
            if let Ok(elapsed) = timeout.started_at.elapsed() {
                if elapsed.as_secs() >= 1 {
                    self.repeat_shortcut_timeout = None;
                    return true;
                }
            }
        }
        false
    }

    // Mode management
    pub fn set_mode_by_index(&mut self, index: Option<usize>) {
        self.mode_state = index;
    }

    pub fn get_current_mode(&self) -> Option<&Mode> {
        self.mode_state.and_then(|index| self.modes.get(index))
    }

    pub fn get_current_mode_name(&self) -> Option<&str> {
        self.get_current_mode().map(|mode| mode.name.as_str())
    }

    pub fn set_modes(&mut self, modes: Vec<Mode>) {
        self.modes = modes;
        self.mode_state = Some(0);
    }
}
