use crate::{
    app::{
        message_state::MessageState,
        ui_components::{
            message_part::VerbosityLevel, FileSelector, MessageLog, SessionSelector, TextInputArea,
        },
    },
    sdk::{
        client::{generate_id, IdPrefix},
        extensions::events::EventStreamHandle,
        OpenCodeClient,
    },
};
use opencode_sdk::models::{AgentConfig, ConfigAgent, File, Session};
use std::{fmt::Display, time::SystemTime};

#[derive(Debug, Clone, PartialEq)]
pub enum RepeatShortcutKey {
    CtrlC,
    CtrlD,
    Esc,
    Leader,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TimeoutType {
    RepeatShortcut(RepeatShortcutKey),
    DebounceFindFiles(String), // query string
}

#[derive(Debug, Clone, PartialEq)]
pub struct Timeout {
    pub timeout_type: TimeoutType,
    pub started_at: SystemTime,
    pub duration_ms: u64,
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
pub struct AttachedFile {
    pub file: File,           // From opencode_sdk::models::File
    pub part_id: String,      // Generated ID for the file part
    pub display_name: String, // For UI display (filename only)
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

#[derive(Debug, Clone)]
pub struct Model {
    pub init: ModelInit,
    pub config: UserConfig,
    // App state
    pub state: AppModalState,
    pub input_history: Vec<String>,
    pub last_input: Option<String>,
    pub printed_to_stdout_count: usize,
    pub sdk_mode: String,
    pub sdk_provider: String,
    pub sdk_model: String,
    // UI state
    pub verbosity_level: VerbosityLevel,
    // Stateful components:
    pub message_log: MessageLog,
    pub text_input_area: TextInputArea, // New tui-textarea based input
    pub modal_session_selector: SessionSelector,
    pub modal_file_selector: FileSelector,
    // Client and session state
    pub client: Option<OpenCodeClient>,
    pub session_state: SessionState,
    pub sessions: Vec<Session>,
    pub modes: Option<ConfigAgent>,
    pub mode_state: Option<u16>,
    pub connection_status: ConnectionStatus,
    pub pending_first_message: Option<String>,
    // Message state and event streaming
    pub message_state: MessageState,
    pub event_stream_state: EventStreamState,
    pub active_task_count: usize,
    // Session state for UI indicators
    pub session_is_idle: bool,
    // File picker state
    pub file_status: Vec<File>,
    // File attachment state
    pub attached_files: Vec<AttachedFile>,
    // Unified repeat shortcut timeout system
    pub repeat_shortcut_timeout: Option<RepeatShortcutTimeout>,
    // General timeout system for debouncing and other purposes
    pub active_timeouts: Vec<Timeout>,
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
    pub ui_status_is_bottom: bool,
    pub ui_status_use_labels: bool,
    pub height: u16,
    pub keys_shortcut_timeout_ms: u16,
}

pub use model_init::ModelInit;

#[derive(Debug, Clone, PartialEq)]
pub enum AppModalState {
    None,
    Connecting(ConnectionStatus),
    ModalHelp,
    ModalFileSelect,
    ModalSessionSelect,
    // SelectModel,
    // SelectAgent,
    // SelectFile,
    // SlashCommands,
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

impl Display for ConnectionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match &self {
                ConnectionStatus::Disconnected => "Disconnected from server! Press 'r' to retry",
                ConnectionStatus::Connecting => "Connecting to OpenCode server...",
                ConnectionStatus::Connected => "Connected to server...",
                ConnectionStatus::InitializingSession => "Initializing session...",
                ConnectionStatus::SessionReady => "✓ Session ready!",
                ConnectionStatus::Error(ref _error) => "Connection failed! Press 'r' to retry",
            }
        );
        Ok(())
    }
}

pub const INLINE_HEIGHT: u16 = 12;

impl Model {
    pub fn new() -> Self {
        let mut text_input_area = TextInputArea::new();
        text_input_area.set_focus(true);

        let message_log = MessageLog::new();
        let modal_session_selector = SessionSelector::new();
        let modal_file_selector = FileSelector::new();

        Model {
            init: ModelInit::new(true),
            config: UserConfig {
                ui_block_is_rounded: true,
                ui_status_is_bottom: true,
                ui_status_use_labels: true,
                height: INLINE_HEIGHT,
                keys_shortcut_timeout_ms: 1000,
            },
            state: AppModalState::Connecting(ConnectionStatus::Connecting),
            input_history: Vec::new(),
            last_input: None,
            printed_to_stdout_count: 0,
            sdk_mode: "chat".to_string(),
            sdk_provider: "anthropic".to_string(),
            sdk_model: "claude-sonnet-4-20250514".to_string(),
            verbosity_level: VerbosityLevel::Summary,
            message_log,
            text_input_area,
            modal_session_selector,
            modal_file_selector,
            client: None,
            session_state: SessionState::None,
            sessions: Vec::new(),
            modes: None,
            mode_state: None,
            connection_status: ConnectionStatus::Connecting,
            pending_first_message: None,
            message_state: MessageState::new(),
            event_stream_state: EventStreamState::Disconnected,
            active_task_count: 0,
            session_is_idle: true,
            file_status: Vec::new(),
            attached_files: Vec::new(),
            repeat_shortcut_timeout: None,
            active_timeouts: Vec::new(),
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

    pub fn message_containers_for_rendering(
        &self,
    ) -> Vec<&crate::app::message_state::MessageContainer> {
        self.message_state.get_message_containers_for_rendering()
    }

    pub fn mark_messages_printed_to_stdout(&mut self, count: usize) {
        self.message_state.mark_messages_printed_to_stdout(count);
        // Keep the old counter for backward compatibility with input_history
        self.printed_to_stdout_count += count;
    }

    // Input management
    pub fn clear_input_state(&mut self) {
        self.text_input_area.clear();
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

    pub fn is_connnection_modal_active(&self) -> bool {
        matches!(
            self.state,
            AppModalState::Connecting(ConnectionStatus::Disconnected)
                | AppModalState::Connecting(ConnectionStatus::InitializingSession)
                | AppModalState::Connecting(ConnectionStatus::Connecting)
                | AppModalState::Connecting(ConnectionStatus::Error(_))
        )
    }

    pub fn is_modal_active(&self) -> bool {
        matches!(
            self.state,
            // Add new modal/overlay states here
            AppModalState::ModalSessionSelect
                | AppModalState::ModalHelp
                | AppModalState::ModalFileSelect
        ) || self.is_connnection_modal_active()
    }

    pub fn is_main_screen_active(&self) -> bool {
        matches!(
            self.state,
            AppModalState::None
                | AppModalState::Connecting(ConnectionStatus::Connected)
                | AppModalState::Connecting(ConnectionStatus::SessionReady)
        ) && !self.is_modal_active()
    }

    pub fn is_session_ready(&self) -> bool {
        self.client.is_some()
            && matches!(self.session_state, SessionState::Ready(_))
            && (matches!(
                self.state,
                AppModalState::Connecting(ConnectionStatus::SessionReady)
            ) || !matches!(self.state, AppModalState::Connecting(_)))
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
        self.message_log.set_message_containers(vec![]);
        self.modal_session_selector.set_current_session_index(index);
        self.state = AppModalState::None;
    }

    pub fn change_session(&mut self, index: Option<usize>) -> bool {
        match index {
            // Handle selection
            Some(0) => {
                self.change_session_by_index(None);
                self.state = AppModalState::None;

                // Create pending session info
                let pending_info = PendingSessionInfo {
                    temp_id: generate_id(IdPrefix::Session),
                    created_at: SystemTime::now(),
                };
                self.session_state = SessionState::Pending(pending_info);
            }
            Some(requested_session_index) => {
                // Use existing session (requested_session_index - 1 in sessions list)
                let session_index = requested_session_index - 1;
                if session_index < self.sessions.len() {
                    self.change_session_by_index(Some(requested_session_index));
                    self.state = AppModalState::Connecting(ConnectionStatus::InitializingSession);

                    return true;
                }
            }
            None => {}
        };
        false
    }

    pub fn current_session_id(&self) -> Option<String> {
        match &self.modal_session_selector.current_session_index() {
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

    pub fn clear_repeat_leader_timeout(&mut self) {
        if matches!(
            self.repeat_shortcut_timeout.clone().map(|m| m.key),
            Some(RepeatShortcutKey::Leader)
        ) {
            self.clear_repeat_shortcut_timeout();
        }
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

    // General timeout management methods
    pub fn set_timeout(&mut self, timeout_type: TimeoutType, duration_ms: u64) {
        // Remove any existing timeout of the same type
        self.active_timeouts.retain(|t| match timeout_type {
            TimeoutType::DebounceFindFiles(_) => {
                !matches!(t.timeout_type, TimeoutType::DebounceFindFiles(_))
            }
            _ => t.timeout_type != timeout_type,
        });

        // Add new timeout
        self.active_timeouts.push(Timeout {
            timeout_type,
            started_at: SystemTime::now(),
            duration_ms,
        });
    }

    pub fn clear_timeout(&mut self, timeout_type: &TimeoutType) {
        self.active_timeouts
            .retain(|t| &t.timeout_type != timeout_type);
    }

    pub fn is_timeout_active(&self, timeout_type: &TimeoutType) -> bool {
        self.active_timeouts.iter().any(|t| {
            &t.timeout_type == timeout_type
                && t.started_at
                    .elapsed()
                    .map(|elapsed| elapsed.as_millis() < t.duration_ms as u128)
                    .unwrap_or(false)
        })
    }

    pub fn get_expired_timeouts(&mut self) -> Vec<TimeoutType> {
        let now = SystemTime::now();
        let mut expired = Vec::new();

        self.active_timeouts.retain(|timeout| {
            if let Ok(elapsed) = timeout.started_at.elapsed() {
                if elapsed.as_millis() >= timeout.duration_ms as u128 {
                    tracing::debug!(
                        "since {:?} has been {} >= {}",
                        timeout.started_at,
                        elapsed.as_millis(),
                        timeout.duration_ms
                    );
                    expired.push(timeout.timeout_type.clone());
                    false // Remove expired timeout
                } else {
                    true // Keep active timeout
                }
            } else {
                false // Remove invalid timeout
            }
        });

        expired
    }

    // Mode management
    pub fn set_mode(&mut self, index: u16) {
        self.mode_state = Some(index);
    }

    pub fn get_current_mode(&self) -> Option<&AgentConfig> {
        match self.mode_state {
            Some(0u16) => self.modes.as_ref().map(|c| c.build.as_ref()).flatten(),
            Some(1u16) => self.modes.as_ref().map(|c| c.plan.as_ref()).flatten(),
            Some(2u16) => self.modes.as_ref().map(|c| c.general.as_ref()).flatten(),
            _ => None,
        }
    }

    pub fn get_current_mode_name(&self) -> Option<String> {
        match self.mode_state {
            Some(0u16) => Some("build"),
            Some(1u16) => Some("plan"),
            Some(2u16) => Some("general"),
            _ => None,
        }
        .map(|m| m.to_string())
    }

    pub fn set_modes(&mut self, modes: ConfigAgent) {
        self.modes = Some(modes);
        self.mode_state = Some(0);
    }

    pub fn increment_mode_index(&mut self) {
        self.mode_state = match self.mode_state {
            None => {
                tracing::debug!("No mode selected, setting to first mode (index 0)");
                Some(0)
            }
            Some(current) => {
                if current > 2 {
                    tracing::debug!(
                        "Current mode index {} out of bounds, resetting to 0",
                        current
                    );
                    Some(0)
                } else {
                    let next = (current + 1) % 3;
                    tracing::debug!("Cycling from mode {} to mode {}", current, next);
                    Some(next)
                }
            }
        };
    }

    pub fn get_mode_and_model_settings(&self) -> (String, String, Option<String>) {
        if let Some(current_mode) = self.get_current_mode() {
            // TODO fix this to be dynamic
            let provider = &self.sdk_provider;
            let model_name = current_mode.model.as_ref().unwrap_or(&self.sdk_model);
            (
                provider.clone(),
                model_name.clone(),
                self.get_current_mode_name(),
            )
        } else {
            // Fallback to hardcoded values if no mode selected
            tracing::debug!("No mode selected for session creation, using fallback provider/model");
            (self.sdk_provider.clone(), self.sdk_model.clone(), None)
        }
    }

    // Verbosity management
    pub fn toggle_verbosity(&mut self) {
        self.verbosity_level = match self.verbosity_level {
            VerbosityLevel::Summary => VerbosityLevel::Verbose,
            VerbosityLevel::Verbose => VerbosityLevel::Summary,
        };
    }
}
