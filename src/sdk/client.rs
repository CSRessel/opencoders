//! High-level client wrapper for the OpenCode API

use crate::sdk::{
    discovery::{discover_opencode_server, DiscoveryConfig},
    error::{OpenCodeError, Result},
    extensions::events::{EventStream, EventStreamHandle},
    LogLevel,
};
use opencode_sdk::{
    apis::{configuration::Configuration, default_api},
    models::{
        AppLogRequest, ConfigAgent, FileRead200Response, FindText200ResponseInner,
        SessionChatRequest, SessionChatRequestPartsInner, SessionMessages200ResponseInner, *,
    },
};
use rand::{thread_rng, Rng};
use reqwest::Client;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

static COUNTER: AtomicU64 = AtomicU64::new(0);
static LAST_TIMESTAMP: AtomicU64 = AtomicU64::new(0);

/// High-level client for the OpenCode API
///
/// This client provides an ergonomic interface to the OpenCode API,
/// wrapping the generated client with additional functionality.
#[derive(Debug, Clone)]
pub struct OpenCodeClient {
    config: Configuration,
    #[allow(dead_code)]
    event_stream: Option<Arc<RwLock<EventStream>>>,
}

#[derive(Debug, Clone, Copy)] // Add traits for convenience
pub enum IdPrefix {
    Message,
    Session,
    User,
    Part,
    Permission,
}
impl IdPrefix {
    pub fn as_str(&self) -> &'static str {
        match self {
            IdPrefix::Message => "msg",
            IdPrefix::Session => "ses",
            IdPrefix::User => "usr",
            IdPrefix::Part => "prt",
            IdPrefix::Permission => "per",
        }
    }
}

pub fn generate_id(prefix: IdPrefix) -> String {
    generate_id_with_direction(prefix, false)
}

pub fn generate_descending_id(prefix: IdPrefix) -> String {
    generate_id_with_direction(prefix, true)
}

fn generate_id_with_direction(prefix: IdPrefix, descending: bool) -> String {
    let current_timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;

    // Handle counter increment with atomic operations to match Go/TypeScript logic
    let (timestamp_to_use, counter) = loop {
        let last_ts = LAST_TIMESTAMP.load(Ordering::SeqCst);

        if current_timestamp != last_ts {
            // Try to update the timestamp and reset counter
            if LAST_TIMESTAMP
                .compare_exchange(
                    last_ts,
                    current_timestamp,
                    Ordering::SeqCst,
                    Ordering::SeqCst,
                )
                .is_ok()
            {
                COUNTER.store(1, Ordering::SeqCst);
                break (current_timestamp, 1);
            }
            // If we failed to update, loop again
        } else {
            // Same timestamp, increment counter
            let counter = COUNTER.fetch_add(1, Ordering::SeqCst) + 1;
            break (current_timestamp, counter);
        }
    };

    // Match TypeScript/Go: (timestamp_ms << 12) + counter
    let mut now = timestamp_to_use * 0x1000 + counter;

    // Apply descending bit flip if requested
    if descending {
        now = !now;
    }

    // Extract 6 bytes like TypeScript/Go (48 bits total)
    let time_bytes: [u8; 6] = [
        ((now >> 40) & 0xff) as u8,
        ((now >> 32) & 0xff) as u8,
        ((now >> 24) & 0xff) as u8,
        ((now >> 16) & 0xff) as u8,
        ((now >> 8) & 0xff) as u8,
        (now & 0xff) as u8,
    ];

    // Convert to hex string (12 hex chars)
    let time_hex = time_bytes
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>();

    // Generate crypto-secure random base62 string (14 chars)
    let mut rng = thread_rng();
    let chars = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
    let random_part: String = (0..14)
        .map(|_| {
            let idx = rng.gen_range(0..62);
            chars.chars().nth(idx).unwrap()
        })
        .collect();

    // Format: {prefix}_{12_hex_chars}{14_base62_chars}
    format!("{}_{}{}", prefix.as_str(), time_hex, random_part)
}

impl OpenCodeClient {
    /// Create a new OpenCode client
    pub fn new(base_url: &str) -> Self {
        let mut config = Configuration::new();
        config.base_path = base_url.to_string();
        config.client = Client::new();

        Self {
            config,
            event_stream: None,
        }
    }

    /// Create a new client with custom HTTP client
    pub fn with_client(base_url: &str, client: Client) -> Self {
        let mut config = Configuration::new();
        config.base_path = base_url.to_string();
        config.client = client;

        Self {
            config,
            event_stream: None,
        }
    }

    /// Discover and connect to a running OpenCode server
    pub async fn discover() -> Result<Self> {
        tracing::info!("Discovering OpenCode server");
        let server_url = discover_opencode_server().await?;
        tracing::info!("Connected to OpenCode server at: {}", server_url);
        Ok(Self::new(&server_url))
    }

    /// Discover and connect to a running OpenCode server with custom configuration
    pub async fn discover_with_config(config: DiscoveryConfig) -> Result<Self> {
        let server_url =
            crate::sdk::discovery::discover_opencode_server_with_config(config).await?;
        Ok(Self::new(&server_url))
    }

    /// Get the base URL this client is connected to
    pub fn base_url(&self) -> &str {
        &self.config.base_path
    }

    /// Get the configuration for this client
    pub fn configuration(&self) -> &Configuration {
        &self.config
    }

    /// Test connection to the server
    pub async fn test_connection(&self) -> Result<()> {
        match self.get_app_info().await {
            Ok(_) => {
                tracing::info!("Connected to OpenCode server at {}", self.base_url());
                Ok(())
            }
            Err(e) => {
                tracing::error!("Failed to connect to server at {}: {}", self.base_url(), e);
                Err(e)
            }
        }
    }

    /// Create a clone of this client (without event stream)
    pub fn clone_client(&self) -> Self {
        Self {
            config: self.config.clone(),
            event_stream: None, // Don't clone event stream
        }
    }

    // App operations

    /// Get application information
    pub async fn get_app_info(&self) -> Result<App> {
        default_api::app_period_get(&self.config)
            .await
            .map_err(OpenCodeError::from)
    }

    /// Initialize the application
    pub async fn initialize_app(&self) -> Result<bool> {
        default_api::app_period_init(&self.config)
            .await
            .map_err(OpenCodeError::from)
    }

    // Configuration operations

    /// Get configuration information
    pub async fn get_config(&self) -> Result<Config> {
        default_api::config_period_get(&self.config)
            .await
            .map_err(OpenCodeError::from)
    }

    /// Get available providers
    pub async fn get_providers(&self) -> Result<ConfigProviders200Response> {
        default_api::config_period_providers(&self.config)
            .await
            .map_err(OpenCodeError::from)
    }

    /// Get available agent configurations (formerly modes)
    pub async fn get_agent_configs(&self) -> Result<ConfigAgent> {
        let config = self.get_config().await?;
        Ok(config.agent.unwrap_or_default())
    }

    // Session operations

    /// Create a new session
    pub async fn create_session(&self) -> Result<Session> {
        let params = default_api::SessionPeriodCreateParams {
            session_create_request: Some(SessionCreateRequest::new()),
        };
        default_api::session_period_create(&self.config, params)
            .await
            .map_err(OpenCodeError::from)
    }

    /// List all sessions
    pub async fn list_sessions(&self) -> Result<Vec<Session>> {
        default_api::session_period_list(&self.config)
            .await
            .map_err(OpenCodeError::from)
    }

    /// Delete a session
    pub async fn delete_session(&self, session_id: &str) -> Result<bool> {
        let params = default_api::SessionPeriodDeleteParams {
            id: session_id.to_string(),
        };
        default_api::session_period_delete(&self.config, params)
            .await
            .map_err(OpenCodeError::from)
    }

    /// Initialize a session (analyze app and create AGENTS.md)
    pub async fn initialize_session(
        &self,
        session_id: &str,
        message_id: &str,
        provider_id: &str,
        model_id: &str,
    ) -> Result<bool> {
        let request = SessionInitRequest {
            message_id: message_id.to_string(),
            provider_id: provider_id.to_string(),
            model_id: model_id.to_string(),
        };

        let params = default_api::SessionPeriodInitParams {
            id: session_id.to_string(),
            session_init_request: Some(request),
        };

        default_api::session_period_init(&self.config, params)
            .await
            .map_err(OpenCodeError::from)
    }

    /// Abort a session
    pub async fn abort_session(&self, session_id: &str) -> Result<bool> {
        let params = default_api::SessionPeriodAbortParams {
            id: session_id.to_string(),
        };
        default_api::session_period_abort(&self.config, params)
            .await
            .map_err(OpenCodeError::from)
    }

    /// Share a session
    pub async fn share_session(&self, session_id: &str) -> Result<Session> {
        let params = default_api::SessionPeriodShareParams {
            id: session_id.to_string(),
        };
        default_api::session_period_share(&self.config, params)
            .await
            .map_err(OpenCodeError::from)
    }

    /// Unshare a session
    pub async fn unshare_session(&self, session_id: &str) -> Result<Session> {
        let params = default_api::SessionPeriodUnshareParams {
            id: session_id.to_string(),
        };
        default_api::session_period_unshare(&self.config, params)
            .await
            .map_err(OpenCodeError::from)
    }

    /// Summarize a session
    pub async fn summarize_session(
        &self,
        session_id: &str,
        provider_id: &str,
        model_id: &str,
    ) -> Result<bool> {
        let request = SessionSummarizeRequest {
            provider_id: provider_id.to_string(),
            model_id: model_id.to_string(),
        };

        let params = default_api::SessionPeriodSummarizeParams {
            id: session_id.to_string(),
            session_summarize_request: Some(request),
        };

        default_api::session_period_summarize(&self.config, params)
            .await
            .map_err(OpenCodeError::from)
    }

    // Message operations

    /// Get messages for a session
    pub async fn get_messages(
        &self,
        session_id: &str,
    ) -> Result<Vec<SessionMessages200ResponseInner>> {
        let params = default_api::SessionPeriodMessagesParams {
            id: session_id.to_string(),
        };

        match default_api::session_period_messages(&self.config, params).await {
            Ok(messages) => {
                tracing::info!(
                    "Retrieved {} messages for session {}",
                    messages.len(),
                    session_id
                );
                Ok(messages)
            }
            Err(e) => {
                tracing::error!("Failed to get messages for session {}: {}", session_id, e);
                Err(OpenCodeError::from(e))
            }
        }
    }

    /// Send a user message to a session
    pub async fn send_user_message(
        &self,
        session_id: &str,
        message_id: &str,
        text: &str,
        provider_id: &str,
        model_id: &str,
        mode: Option<&str>,
    ) -> Result<AssistantMessage> {
        tracing::info!("Sending message to session {}", session_id);

        let text_part = TextPartInput {
            id: Some(generate_id(IdPrefix::Part)),
            text: text.to_string(),
            synthetic: None,
            time: None,
        };

        let part = SessionChatRequestPartsInner::Text(Box::new(text_part));
        let request = SessionChatRequest {
            message_id: Some(message_id.to_string()),
            provider_id: provider_id.to_string(),
            model_id: model_id.to_string(),
            agent: mode.map(|m| m.to_string()),
            system: None,
            tools: None,
            parts: vec![part],
        };

        let params = default_api::SessionPeriodChatParams {
            id: session_id.to_string(),
            session_chat_request: Some(request),
        };

        match default_api::session_period_chat(&self.config, params).await {
            Ok(message) => {
                tracing::info!("Message sent successfully");
                Ok(message)
            }
            Err(e) => {
                tracing::error!("Failed to send message: {}", e);
                Err(OpenCodeError::from(e))
            }
        }
    }

    /// Create a message builder for complex message construction
    pub fn message_builder(&self, session_id: &str) -> MessageBuilder {
        MessageBuilder::new(session_id)
    }

    // File operations

    /// Read a file
    pub async fn read_file(&self, path: &str) -> Result<FileRead200Response> {
        let params = default_api::FilePeriodReadParams {
            path: path.to_string(),
        };
        default_api::file_period_read(&self.config, params)
            .await
            .map_err(OpenCodeError::from)
    }

    /// Get file status
    pub async fn get_file_status(&self) -> Result<Vec<File>> {
        default_api::file_period_status(&self.config)
            .await
            .map_err(OpenCodeError::from)
    }

    // Search operations

    /// Find text in files
    pub async fn find_text(&self, pattern: &str) -> Result<Vec<FindText200ResponseInner>> {
        let params = default_api::FindPeriodTextParams {
            pattern: pattern.to_string(),
        };
        default_api::find_period_text(&self.config, params)
            .await
            .map_err(OpenCodeError::from)
    }

    /// Find files
    pub async fn find_files(&self, query: &str) -> Result<Vec<String>> {
        let params = default_api::FindPeriodFilesParams {
            query: query.to_string(),
        };
        default_api::find_period_files(&self.config, params)
            .await
            .map_err(OpenCodeError::from)
    }

    /// Find symbols
    pub async fn find_symbols(&self, query: &str) -> Result<Vec<Symbol>> {
        let params = default_api::FindPeriodSymbolsParams {
            query: query.to_string(),
        };
        default_api::find_period_symbols(&self.config, params)
            .await
            .map_err(OpenCodeError::from)
    }

    // Logging

    /// Write a log entry
    pub async fn write_log(
        &self,
        service: &str,
        level: LogLevel,
        message: &str,
        extra: Option<std::collections::HashMap<String, serde_json::Value>>,
    ) -> Result<bool> {
        // Convert LogLevel to the Level enum expected by AppLogRequest
        let app_log_level = match level {
            LogLevel::Debug => app_log_request::Level::Debug,
            LogLevel::Info => app_log_request::Level::Info,
            LogLevel::Warn => app_log_request::Level::Warn,
            LogLevel::Error => app_log_request::Level::Error,
        };

        let request = AppLogRequest {
            service: service.to_string(),
            level: app_log_level,
            message: message.to_string(),
            extra,
        };

        let params = default_api::AppPeriodLogParams {
            app_log_request: Some(request),
        };

        default_api::app_period_log(&self.config, params)
            .await
            .map_err(OpenCodeError::from)
    }

    // Event streaming

    /// Subscribe to real-time events
    pub async fn subscribe_to_events(&mut self) -> Result<EventStreamHandle> {
        let stream = EventStream::new(self.config.clone()).await?;
        let handle = stream.handle();
        self.event_stream = Some(Arc::new(RwLock::new(stream)));
        Ok(handle)
    }
}

impl PartialEq for OpenCodeClient {
    fn eq(&self, other: &Self) -> bool {
        self.config.base_path == other.config.base_path
    }
}

/// Builder for constructing complex message requests
#[derive(Debug, Clone)]
pub struct MessageBuilder {
    session_id: String,
    message_id: Option<String>,
    provider_id: Option<String>,
    model_id: Option<String>,
    mode: Option<String>,
    parts: Vec<SessionChatRequestPartsInner>,
}

impl MessageBuilder {
    fn new(session_id: &str) -> Self {
        Self {
            session_id: session_id.to_string(),
            message_id: None,
            provider_id: None,
            model_id: None,
            mode: None,
            parts: Vec::new(),
        }
    }

    /// Set the message ID
    pub fn message_id(mut self, id: &str) -> Self {
        self.message_id = Some(id.to_string());
        self
    }

    /// Set the provider ID
    pub fn provider(mut self, provider_id: &str) -> Self {
        self.provider_id = Some(provider_id.to_string());
        self
    }

    /// Set the model ID
    pub fn model(mut self, model_id: &str) -> Self {
        self.model_id = Some(model_id.to_string());
        self
    }

    /// Set the mode
    pub fn mode(mut self, mode: &str) -> Self {
        self.mode = Some(mode.to_string());
        self
    }

    /// Add a text part to the message
    pub fn add_text_part(mut self, text: &str) -> Self {
        let text_part = TextPartInput {
            id: Some(generate_id(IdPrefix::Part)),
            text: text.to_string(),
            synthetic: None,
            time: None,
        };
        let part = SessionChatRequestPartsInner::Text(Box::new(text_part));
        self.parts.push(part);
        self
    }

    /// Add a file part to the message
    pub fn add_file_part(mut self, filename: &str, mime: &str, url: &str) -> Self {
        let file_part = FilePartInput {
            id: Some(generate_id(IdPrefix::Part)),
            mime: mime.to_string(),
            filename: Some(filename.to_string()),
            url: url.to_string(),
            source: None,
        };
        let part = SessionChatRequestPartsInner::File(Box::new(file_part));
        self.parts.push(part);
        self
    }

    /// Send the message
    pub async fn send(self, config: &Configuration) -> Result<AssistantMessage> {
        let request = SessionChatRequest {
            message_id: Some(
                self.message_id
                    .ok_or_else(|| OpenCodeError::invalid_request("message_id is required"))?,
            ),
            provider_id: self
                .provider_id
                .ok_or_else(|| OpenCodeError::invalid_request("provider_id is required"))?,
            model_id: self
                .model_id
                .ok_or_else(|| OpenCodeError::invalid_request("model_id is required"))?,
            agent: self.mode,
            system: None,
            tools: None,
            parts: self.parts,
        };

        let params = default_api::SessionPeriodChatParams {
            id: self.session_id,
            session_chat_request: Some(request),
        };

        default_api::session_period_chat(config, params)
            .await
            .map_err(OpenCodeError::from)
    }
}
