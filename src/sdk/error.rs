//! Error types for the OpenCode SDK

use opencode_sdk::apis;
use thiserror::Error;



/// Result type alias for OpenCode SDK operations
pub type Result<T> = std::result::Result<T, OpenCodeError>;

/// Main error type for the OpenCode SDK
#[derive(Error, Debug)]
pub enum OpenCodeError {
    /// HTTP request failed
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    /// JSON serialization/deserialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// API returned an error response
    #[error("API error: {status} - {message}")]
    Api { status: u16, message: String },

    /// Authentication/authorization error
    #[error("Authentication error: {0}")]
    Auth(String),

    /// Session not found
    #[error("Session not found: {session_id}")]
    SessionNotFound { session_id: String },

    /// Message not found
    #[error("Message not found: {message_id} in session {session_id}")]
    MessageNotFound {
        session_id: String,
        message_id: String,
    },

    /// Event stream error
    #[error("Event stream error: {0}")]
    EventStream(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Invalid request parameters
    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    /// Timeout error
    #[error("Request timeout: {0}")]
    Timeout(String),

    /// Server not found during discovery
    #[error("OpenCode server not found - check if server is running")]
    ServerNotFound,

    /// Connection timeout
    #[error("Connection timeout")]
    ConnectionTimeout,

    /// Process detection failed
    #[error("Failed to detect running OpenCode processes")]
    ProcessDetectionFailed,

    /// Session persistence error
    #[error("Session persistence error: {0}")]
    SessionPersistence(String),

    /// Server start failed
    #[error("Failed to start OpenCode server: {0}")]
    ServerStartFailed(String),

    /// Generic error for unexpected situations
    #[error("Unexpected error: {0}")]
    Unexpected(String),
}

impl Clone for OpenCodeError {
    fn clone(&self) -> Self {
        match self {
            // Convert non-cloneable errors to Unexpected with preserved error message
            Self::Http(e) => Self::Unexpected(format!("HTTP error: {}", e)),
            Self::Serialization(e) => Self::Unexpected(format!("Serialization error: {}", e)),
            // All other variants can be cloned normally
            Self::Api { status, message } => Self::Api { status: *status, message: message.clone() },
            Self::Auth(msg) => Self::Auth(msg.clone()),
            Self::SessionNotFound { session_id } => Self::SessionNotFound { session_id: session_id.clone() },
            Self::MessageNotFound { session_id, message_id } => Self::MessageNotFound { 
                session_id: session_id.clone(), 
                message_id: message_id.clone() 
            },
            Self::EventStream(msg) => Self::EventStream(msg.clone()),
            Self::Configuration(msg) => Self::Configuration(msg.clone()),
            Self::InvalidRequest(msg) => Self::InvalidRequest(msg.clone()),
            Self::Timeout(msg) => Self::Timeout(msg.clone()),
            Self::ServerNotFound => Self::ServerNotFound,
            Self::ConnectionTimeout => Self::ConnectionTimeout,
            Self::ProcessDetectionFailed => Self::ProcessDetectionFailed,
            Self::SessionPersistence(msg) => Self::SessionPersistence(msg.clone()),
            Self::ServerStartFailed(msg) => Self::ServerStartFailed(msg.clone()),
            Self::Unexpected(msg) => Self::Unexpected(msg.clone()),
        }
    }
}

impl PartialEq for OpenCodeError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            // Non-comparable errors - compare by error message string representation
            (Self::Http(a), Self::Http(b)) => a.to_string() == b.to_string(),
            (Self::Serialization(a), Self::Serialization(b)) => a.to_string() == b.to_string(),
            // Comparable variants
            (Self::Api { status: s1, message: m1 }, Self::Api { status: s2, message: m2 }) => s1 == s2 && m1 == m2,
            (Self::Auth(a), Self::Auth(b)) => a == b,
            (Self::SessionNotFound { session_id: a }, Self::SessionNotFound { session_id: b }) => a == b,
            (Self::MessageNotFound { session_id: s1, message_id: m1 }, Self::MessageNotFound { session_id: s2, message_id: m2 }) => s1 == s2 && m1 == m2,
            (Self::EventStream(a), Self::EventStream(b)) => a == b,
            (Self::Configuration(a), Self::Configuration(b)) => a == b,
            (Self::InvalidRequest(a), Self::InvalidRequest(b)) => a == b,
            (Self::Timeout(a), Self::Timeout(b)) => a == b,
            (Self::ServerNotFound, Self::ServerNotFound) => true,
            (Self::ConnectionTimeout, Self::ConnectionTimeout) => true,
            (Self::ProcessDetectionFailed, Self::ProcessDetectionFailed) => true,
            (Self::SessionPersistence(a), Self::SessionPersistence(b)) => a == b,
            (Self::ServerStartFailed(a), Self::ServerStartFailed(b)) => a == b,
            (Self::Unexpected(a), Self::Unexpected(b)) => a == b,
            _ => false,
        }
    }
}

impl OpenCodeError {
    /// Create an API error from status code and message
    pub fn api_error(status: u16, message: impl Into<String>) -> Self {
        Self::Api {
            status,
            message: message.into(),
        }
    }

    /// Create an authentication error
    pub fn auth_error(message: impl Into<String>) -> Self {
        Self::Auth(message.into())
    }

    /// Create a session not found error
    pub fn session_not_found(session_id: impl Into<String>) -> Self {
        Self::SessionNotFound {
            session_id: session_id.into(),
        }
    }

    /// Create a message not found error
    pub fn message_not_found(session_id: impl Into<String>, message_id: impl Into<String>) -> Self {
        Self::MessageNotFound {
            session_id: session_id.into(),
            message_id: message_id.into(),
        }
    }

    /// Create an event stream error
    pub fn event_stream_error(message: impl Into<String>) -> Self {
        Self::EventStream(message.into())
    }

    /// Create a configuration error
    pub fn configuration_error(message: impl Into<String>) -> Self {
        Self::Configuration(message.into())
    }

    /// Create an invalid request error
    pub fn invalid_request(message: impl Into<String>) -> Self {
        Self::InvalidRequest(message.into())
    }

    /// Create a timeout error
    pub fn timeout_error(message: impl Into<String>) -> Self {
        Self::Timeout(message.into())
    }

    /// Create a session persistence error
    pub fn session_persistence_error(message: impl Into<String>) -> Self {
        Self::SessionPersistence(message.into())
    }

    /// Create a server start failed error
    pub fn server_start_failed(message: impl Into<String>) -> Self {
        Self::ServerStartFailed(message.into())
    }

    /// Check if this error is retryable
    pub fn is_retryable(&self) -> bool {
        match self {
            Self::Http(e) => e.is_timeout() || e.is_connect(),
            Self::Api { status, .. } => *status >= 500,
            Self::Timeout(_) => true,
            Self::EventStream(_) => true,
            Self::ConnectionTimeout => true,
            Self::ProcessDetectionFailed => true,
            Self::ServerStartFailed(_) => false,
            _ => false,
        }
    }

    /// Check if this error is a client error (4xx)
    pub fn is_client_error(&self) -> bool {
        match self {
            Self::Api { status, .. } => *status >= 400 && *status < 500,
            Self::Auth(_) => true,
            Self::SessionNotFound { .. } => true,
            Self::MessageNotFound { .. } => true,
            Self::InvalidRequest(_) => true,
            _ => false,
        }
    }

    /// Check if this error is a server error (5xx)
    pub fn is_server_error(&self) -> bool {
        match self {
            Self::Api { status, .. } => *status >= 500,
            _ => false,
        }
    }
}

// Generic From implementation for generated API errors
impl<T> From<apis::Error<T>> for OpenCodeError {
    fn from(error: apis::Error<T>) -> Self {
        match error {
            apis::Error::Reqwest(e) => OpenCodeError::Http(e),
            apis::Error::Serde(e) => OpenCodeError::Serialization(e),
            apis::Error::Io(e) => OpenCodeError::Unexpected(e.to_string()),
            apis::Error::ResponseError(response) => OpenCodeError::Api {
                status: response.status.as_u16(),
                message: response.content,
            },
        }
    }
}

