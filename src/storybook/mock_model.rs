use crate::app::{
    tea_model::{UserConfig, INLINE_HEIGHT},
    ui_components::{message_part::VerbosityLevel, MessageLog, SessionSelector, TextInputArea},
};

#[derive(Debug, Clone)]
pub struct MockModel {
    pub config: UserConfig,
    pub verbosity_level: VerbosityLevel,
    pub message_log: MessageLog,
    pub text_input_area: TextInputArea,
    pub session_selector: SessionSelector,

    // Mock simple states
    pub mock_session_ready: bool,
    pub mock_client_ready: bool,
}

impl MockModel {
    pub fn new() -> Self {
        Self {
            config: UserConfig {
                ui_block_is_rounded: false,
                ui_status_is_bottom: true,
                ui_status_use_labels: true,
                height: INLINE_HEIGHT,
                keys_shortcut_timeout_ms: 1000,
            },
            verbosity_level: VerbosityLevel::Summary,
            message_log: MessageLog::new(),
            text_input_area: TextInputArea::with_placeholder("Mock input..."),
            session_selector: SessionSelector::new(),
            mock_session_ready: true,
            mock_client_ready: true,
        }
    }

    // Mock methods that would trigger side effects - print to terminal instead
    pub fn mock_submit_message(&self, message: &str) {
        println!("[MOCK SIDE EFFECT] Would submit message: {}", message);
    }

    pub fn mock_create_session(&self) {
        println!("[MOCK SIDE EFFECT] Would create new session");
    }

    pub fn mock_connect_to_server(&self) {
        println!("[MOCK SIDE EFFECT] Would connect to server");
    }

    pub fn mock_toggle_session_selector(&self) {
        println!("[MOCK SIDE EFFECT] Would toggle session selector");
    }

    // Simple state getters for UI components
    pub fn is_session_ready(&self) -> bool {
        self.mock_session_ready
    }

    pub fn is_client_ready(&self) -> bool {
        self.mock_client_ready
    }

    pub fn can_accept_input(&self) -> bool {
        true
    }

    // Method to execute mock model within ViewModelContext safely
    pub fn with_context<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        // For now, we'll skip the ViewModelContext since it requires a full Model
        // In a real implementation, we'd need to adapt ViewModelContext for MockModel
        f()
    }
}
