//! Error types for the OpenCode SDK

use opencode_sdk::apis;
use thiserror::Error;

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

    /// Generic error for unexpected situations
    #[error("Unexpected error: {0}")]
    Unexpected(String),
}

/// Result type alias for OpenCode SDK operations
pub type Result<T> = std::result::Result<T, OpenCodeError>;

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

    /// Check if this error is retryable
    pub fn is_retryable(&self) -> bool {
        match self {
            Self::Http(e) => e.is_timeout() || e.is_connect(),
            Self::Api { status, .. } => *status >= 500,
            Self::Timeout(_) => true,
            Self::EventStream(_) => true,
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

