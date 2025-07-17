//! High-level client wrapper for the OpenCode API

use crate::sdk::{
    error::{OpenCodeError, Result},
    extensions::events::{EventStream, EventStreamHandle},
    generated::{
        apis::{configuration::Configuration, DefaultApi},
        models::*,
    },
};
use reqwest::Client;
use std::sync::Arc;
use tokio::sync::RwLock;

/// High-level client for the OpenCode API
/// 
/// This client provides an ergonomic interface to the OpenCode API,
/// wrapping the generated client with additional functionality.
pub struct OpenCodeClient {
    config: Configuration,
    api: DefaultApi,
    event_stream: Option<Arc<RwLock<EventStream>>>,
}

impl OpenCodeClient {
    /// Create a new OpenCode client
    pub fn new(base_url: &str) -> Self {
        let mut config = Configuration::new();
        config.base_path = base_url.to_string();
        config.client = Client::new();
        
        let api = DefaultApi::new(config.clone());
        
        Self {
            config,
            api,
            event_stream: None,
        }
    }
    
    /// Create a new client with custom HTTP client
    pub fn with_client(base_url: &str, client: Client) -> Self {
        let mut config = Configuration::new();
        config.base_path = base_url.to_string();
        config.client = client;
        
        let api = DefaultApi::new(config.clone());
        
        Self {
            config,
            api,
            event_stream: None,
        }
    }
    
    // App operations
    
    /// Get application information
    pub async fn get_app_info(&self) -> Result<App> {
        self.api.get_app().await.map_err(OpenCodeError::from)
    }
    
    /// Initialize the application
    pub async fn initialize_app(&self) -> Result<bool> {
        self.api.post_app_init().await.map_err(OpenCodeError::from)
    }
    
    // Configuration operations
    
    /// Get configuration information
    pub async fn get_config(&self) -> Result<Config> {
        self.api.get_config().await.map_err(OpenCodeError::from)
    }
    
    /// Get available providers
    pub async fn get_providers(&self) -> Result<GetConfigProviders200Response> {
        self.api.get_config_providers().await.map_err(OpenCodeError::from)
    }
    
    /// Get available modes
    pub async fn get_modes(&self) -> Result<Vec<Mode>> {
        self.api.get_mode().await.map_err(OpenCodeError::from)
    }
    
    // Session operations
    
    /// Create a new session
    pub async fn create_session(&self) -> Result<Session> {
        self.api.post_session().await.map_err(OpenCodeError::from)
    }
    
    /// List all sessions
    pub async fn list_sessions(&self) -> Result<Vec<Session>> {
        self.api.get_session().await.map_err(OpenCodeError::from)
    }
    
    /// Delete a session
    pub async fn delete_session(&self, session_id: &str) -> Result<bool> {
        self.api
            .delete_session_by_id(session_id)
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
        
        self.api
            .post_session_by_id_init(session_id, request)
            .await
            .map_err(OpenCodeError::from)
    }
    
    /// Abort a session
    pub async fn abort_session(&self, session_id: &str) -> Result<bool> {
        self.api
            .post_session_by_id_abort(session_id)
            .await
            .map_err(OpenCodeError::from)
    }
    
    /// Share a session
    pub async fn share_session(&self, session_id: &str) -> Result<Session> {
        self.api
            .post_session_by_id_share(session_id)
            .await
            .map_err(OpenCodeError::from)
    }
    
    /// Unshare a session
    pub async fn unshare_session(&self, session_id: &str) -> Result<Session> {
        self.api
            .delete_session_by_id_share(session_id)
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
        
        self.api
            .post_session_by_id_summarize(session_id, request)
            .await
            .map_err(OpenCodeError::from)
    }
    
    // Message operations
    
    /// Get messages for a session
    pub async fn get_messages(&self, session_id: &str) -> Result<Vec<GetSessionByIdMessage200ResponseInner>> {
        self.api
            .get_session_by_id_message(session_id)
            .await
            .map_err(OpenCodeError::from)
    }
    
    /// Create a message builder for complex message construction
    pub fn message_builder(&self, session_id: &str) -> MessageBuilder {
        MessageBuilder::new(self.api.clone(), session_id)
    }
    
    // File operations
    
    /// Read a file
    pub async fn read_file(&self, path: &str) -> Result<GetFile200Response> {
        self.api.get_file(path).await.map_err(OpenCodeError::from)
    }
    
    /// Get file status
    pub async fn get_file_status(&self) -> Result<Vec<File>> {
        self.api.get_file_status().await.map_err(OpenCodeError::from)
    }
    
    // Search operations
    
    /// Find text in files
    pub async fn find_text(&self, pattern: &str) -> Result<Vec<Match>> {
        self.api.get_find(pattern).await.map_err(OpenCodeError::from)
    }
    
    /// Find files
    pub async fn find_files(&self, query: &str) -> Result<Vec<String>> {
        self.api.get_find_file(query).await.map_err(OpenCodeError::from)
    }
    
    /// Find symbols
    pub async fn find_symbols(&self, query: &str) -> Result<Vec<Symbol>> {
        self.api.get_find_symbol(query).await.map_err(OpenCodeError::from)
    }
    
    // Logging
    
    /// Write a log entry
    pub async fn write_log(
        &self,
        service: &str,
        level: &str,
        message: &str,
        extra: Option<serde_json::Value>,
    ) -> Result<bool> {
        let request = PostLogRequest {
            service: service.to_string(),
            level: level.to_string(),
            message: message.to_string(),
            extra,
        };
        
        self.api.post_log(request).await.map_err(OpenCodeError::from)
    }
    
    // Event streaming
    
    /// Subscribe to real-time events
    pub async fn subscribe_to_events(&mut self) -> Result<EventStreamHandle> {
        let stream = EventStream::new(self.api.clone()).await?;
        let handle = stream.handle();
        self.event_stream = Some(Arc::new(RwLock::new(stream)));
        Ok(handle)
    }
}

/// Builder for constructing complex message requests
pub struct MessageBuilder {
    api: DefaultApi,
    session_id: String,
    message_id: Option<String>,
    provider_id: Option<String>,
    model_id: Option<String>,
    mode: Option<String>,
    parts: Vec<PostSessionByIdMessageRequestPartsInner>,
}

impl MessageBuilder {
    fn new(api: DefaultApi, session_id: &str) -> Self {
        Self {
            api,
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
        // Note: The exact structure will depend on the generated types
        // This is a placeholder that will need to be adjusted based on the actual generated code
        self.parts.push(PostSessionByIdMessageRequestPartsInner::TextPart(
            TextPart {
                id: uuid::Uuid::new_v4().to_string(),
                session_id: self.session_id.clone(),
                message_id: self.message_id.clone().unwrap_or_default(),
                r#type: "text".to_string(),
                text: text.to_string(),
                synthetic: None,
                time: None,
            }
        ));
        self
    }
    
    /// Add a file part to the message
    pub fn add_file_part(mut self, filename: &str, mime: &str, url: &str) -> Self {
        self.parts.push(PostSessionByIdMessageRequestPartsInner::FilePart(
            FilePart {
                id: uuid::Uuid::new_v4().to_string(),
                session_id: self.session_id.clone(),
                message_id: self.message_id.clone().unwrap_or_default(),
                r#type: "file".to_string(),
                mime: mime.to_string(),
                filename: Some(filename.to_string()),
                url: url.to_string(),
            }
        ));
        self
    }
    
    /// Send the message
    pub async fn send(self) -> Result<AssistantMessage> {
        let request = PostSessionByIdMessageRequest {
            message_id: self.message_id.ok_or_else(|| {
                OpenCodeError::invalid_request("message_id is required")
            })?,
            provider_id: self.provider_id.ok_or_else(|| {
                OpenCodeError::invalid_request("provider_id is required")
            })?,
            model_id: self.model_id.ok_or_else(|| {
                OpenCodeError::invalid_request("model_id is required")
            })?,
            mode: self.mode.ok_or_else(|| {
                OpenCodeError::invalid_request("mode is required")
            })?,
            parts: self.parts,
        };
        
        self.api
            .post_session_by_id_message(&self.session_id, request)
            .await
            .map_err(OpenCodeError::from)
    }
}