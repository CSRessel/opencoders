//! High-level client wrapper for the OpenCode API

use crate::sdk::{
    discovery::{discover_opencode_server, DiscoveryConfig},
    error::{OpenCodeError, Result},
    extensions::events::{EventStream, EventStreamHandle},
    LogLevel,
};
use opencode_sdk::{
    apis::{configuration::Configuration, default_api},
    models::{post_log_request, *},
};
use reqwest::Client;
use std::sync::Arc;
use tokio::sync::RwLock;

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
        let server_url = discover_opencode_server().await?;
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

    /// Test connection to the server
    pub async fn test_connection(&self) -> Result<()> {
        self.get_app_info().await.map(|_| ())
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
        default_api::get_app(&self.config)
            .await
            .map_err(OpenCodeError::from)
    }

    /// Initialize the application
    pub async fn initialize_app(&self) -> Result<bool> {
        default_api::post_app_init(&self.config)
            .await
            .map_err(OpenCodeError::from)
    }

    // Configuration operations

    /// Get configuration information
    pub async fn get_config(&self) -> Result<Config> {
        default_api::get_config(&self.config)
            .await
            .map_err(OpenCodeError::from)
    }

    /// Get available providers
    pub async fn get_providers(&self) -> Result<GetConfigProviders200Response> {
        default_api::get_config_providers(&self.config)
            .await
            .map_err(OpenCodeError::from)
    }

    /// Get available modes
    pub async fn get_modes(&self) -> Result<Vec<Mode>> {
        default_api::get_mode(&self.config)
            .await
            .map_err(OpenCodeError::from)
    }

    // Session operations

    /// Create a new session
    pub async fn create_session(&self) -> Result<Session> {
        default_api::post_session(&self.config)
            .await
            .map_err(OpenCodeError::from)
    }

    /// List all sessions
    pub async fn list_sessions(&self) -> Result<Vec<Session>> {
        default_api::get_session(&self.config)
            .await
            .map_err(OpenCodeError::from)
    }

    /// Delete a session
    pub async fn delete_session(&self, session_id: &str) -> Result<bool> {
        let params = default_api::DeleteSessionByIdParams {
            id: session_id.to_string(),
        };
        default_api::delete_session_by_id(&self.config, params)
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
        let request = PostSessionByIdInitRequest {
            message_id: message_id.to_string(),
            provider_id: provider_id.to_string(),
            model_id: model_id.to_string(),
        };

        let params = default_api::PostSessionByIdInitParams {
            id: session_id.to_string(),
            post_session_by_id_init_request: Some(request),
        };

        default_api::post_session_by_id_init(&self.config, params)
            .await
            .map_err(OpenCodeError::from)
    }

    /// Abort a session
    pub async fn abort_session(&self, session_id: &str) -> Result<bool> {
        let params = default_api::PostSessionByIdAbortParams {
            id: session_id.to_string(),
        };
        default_api::post_session_by_id_abort(&self.config, params)
            .await
            .map_err(OpenCodeError::from)
    }

    /// Share a session
    pub async fn share_session(&self, session_id: &str) -> Result<Session> {
        let params = default_api::PostSessionByIdShareParams {
            id: session_id.to_string(),
        };
        default_api::post_session_by_id_share(&self.config, params)
            .await
            .map_err(OpenCodeError::from)
    }

    /// Unshare a session
    pub async fn unshare_session(&self, session_id: &str) -> Result<Session> {
        let params = default_api::DeleteSessionByIdShareParams {
            id: session_id.to_string(),
        };
        default_api::delete_session_by_id_share(&self.config, params)
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
        let request = PostSessionByIdSummarizeRequest {
            provider_id: provider_id.to_string(),
            model_id: model_id.to_string(),
        };

        let params = default_api::PostSessionByIdSummarizeParams {
            id: session_id.to_string(),
            post_session_by_id_summarize_request: Some(request),
        };

        default_api::post_session_by_id_summarize(&self.config, params)
            .await
            .map_err(OpenCodeError::from)
    }

    // Message operations

    /// Get messages for a session
    pub async fn get_messages(
        &self,
        session_id: &str,
    ) -> Result<Vec<GetSessionByIdMessage200ResponseInner>> {
        let params = default_api::GetSessionByIdMessageParams {
            id: session_id.to_string(),
        };
        default_api::get_session_by_id_message(&self.config, params)
            .await
            .map_err(OpenCodeError::from)
    }

    /// Create a message builder for complex message construction
    pub fn message_builder(&self, session_id: &str) -> MessageBuilder {
        MessageBuilder::new(session_id)
    }

    // File operations

    /// Read a file
    pub async fn read_file(&self, path: &str) -> Result<GetFile200Response> {
        let params = default_api::GetFileParams {
            path: path.to_string(),
        };
        default_api::get_file(&self.config, params)
            .await
            .map_err(OpenCodeError::from)
    }

    /// Get file status
    pub async fn get_file_status(&self) -> Result<Vec<File>> {
        default_api::get_file_status(&self.config)
            .await
            .map_err(OpenCodeError::from)
    }

    // Search operations

    /// Find text in files
    pub async fn find_text(&self, pattern: &str) -> Result<Vec<Match>> {
        let params = default_api::GetFindParams {
            pattern: pattern.to_string(),
        };
        default_api::get_find(&self.config, params)
            .await
            .map_err(OpenCodeError::from)
    }

    /// Find files
    pub async fn find_files(&self, query: &str) -> Result<Vec<String>> {
        let params = default_api::GetFindFileParams {
            query: query.to_string(),
        };
        default_api::get_find_file(&self.config, params)
            .await
            .map_err(OpenCodeError::from)
    }

    /// Find symbols
    pub async fn find_symbols(&self, query: &str) -> Result<Vec<Symbol>> {
        let params = default_api::GetFindSymbolParams {
            query: query.to_string(),
        };
        default_api::get_find_symbol(&self.config, params)
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
        // Convert LogLevel to the Level enum expected by PostLogRequest
        let post_log_level = match level {
            LogLevel::Debug => post_log_request::Level::Debug,
            LogLevel::Info => post_log_request::Level::Info,
            LogLevel::Warn => post_log_request::Level::Warn,
            LogLevel::Error => post_log_request::Level::Error,
        };

        let request = PostLogRequest {
            service: service.to_string(),
            level: post_log_level,
            message: message.to_string(),
            extra,
        };

        let params = default_api::PostLogParams {
            post_log_request: Some(request),
        };

        default_api::post_log(&self.config, params)
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
    parts: Vec<PostSessionByIdMessageRequestPartsInner>,
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
            id: Some(uuid::Uuid::new_v4().to_string()),
            r#type: "text".to_string(),
            text: text.to_string(),
            synthetic: None,
            time: None,
        };
        let part = PostSessionByIdMessageRequestPartsInner::Text(Box::new(text_part));
        self.parts.push(part);
        self
    }

    /// Add a file part to the message
    pub fn add_file_part(mut self, filename: &str, mime: &str, url: &str) -> Self {
        let file_part = FilePartInput {
            id: Some(uuid::Uuid::new_v4().to_string()),
            r#type: "file".to_string(),
            mime: mime.to_string(),
            filename: Some(filename.to_string()),
            url: url.to_string(),
            source: None,
        };
        let part = PostSessionByIdMessageRequestPartsInner::File(Box::new(file_part));
        self.parts.push(part);
        self
    }

    /// Send the message
    pub async fn send(self, config: &Configuration) -> Result<AssistantMessage> {
        let request = PostSessionByIdMessageRequest {
            message_id: Some(self
                .message_id
                .ok_or_else(|| OpenCodeError::invalid_request("message_id is required"))?),
            provider_id: self
                .provider_id
                .ok_or_else(|| OpenCodeError::invalid_request("provider_id is required"))?,
            model_id: self
                .model_id
                .ok_or_else(|| OpenCodeError::invalid_request("model_id is required"))?,
            mode: Some(self
                .mode
                .ok_or_else(|| OpenCodeError::invalid_request("mode is required"))?),
            tools: None,
            parts: self.parts,
        };

        let params = default_api::PostSessionByIdMessageParams {
            id: self.session_id,
            post_session_by_id_message_request: Some(request),
        };

        default_api::post_session_by_id_message(config, params)
            .await
            .map_err(OpenCodeError::from)
    }
}
