//! Session persistence and management utilities
//!
//! This module provides functionality to persist session state locally
//! and manage session lifecycle for the TUI application.

use crate::sdk::{error::{OpenCodeError, Result}, OpenCodeClient};
use opencode_sdk::models::Session;
use std::path::PathBuf;
use std::env;
use tokio::fs;

/// Session manager for handling session persistence and lifecycle
pub struct SessionManager {
    client: OpenCodeClient,
    state_dir: PathBuf,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new(client: OpenCodeClient) -> Self {
        let state_dir = get_opencode_state_dir();
        Self { client, state_dir }
    }

    /// Get or create a session, preferring to reuse the last session if valid
    pub async fn get_or_create_session(&self) -> Result<Session> {
        // 1. Try to load last session from local storage
        if let Ok(session_id) = self.load_last_session_id().await {
            // 2. Validate session still exists
            if let Ok(sessions) = self.client.list_sessions().await {
                if let Some(session) = sessions.into_iter().find(|s| s.id == session_id) {
                    return Ok(session);
                }
            }
        }

        // 3. Create new session if none exists or invalid
        let session = self.client.create_session().await?;
        self.save_last_session_id(&session.id).await?;
        Ok(session)
    }

    /// Create a new session and save it as the current session
    pub async fn create_new_session(&self) -> Result<Session> {
        let session = self.client.create_session().await?;
        self.save_last_session_id(&session.id).await?;
        Ok(session)
    }

    /// Get the current session ID if one exists and is valid
    pub async fn get_current_session_id(&self) -> Option<String> {
        if let Ok(session_id) = self.load_last_session_id().await {
            // Validate session still exists
            if let Ok(sessions) = self.client.list_sessions().await {
                if sessions.iter().any(|s| s.id == session_id) {
                    return Some(session_id);
                }
            }
        }
        None
    }

    /// Switch to a specific session
    pub async fn switch_to_session(&self, session_id: &str) -> Result<Session> {
        // Validate session exists
        let sessions = self.client.list_sessions().await?;
        let session = sessions
            .into_iter()
            .find(|s| s.id == session_id)
            .ok_or_else(|| OpenCodeError::session_not_found(session_id))?;

        // Save as current session
        self.save_last_session_id(session_id).await?;
        Ok(session)
    }

    /// Clear the current session (forces creation of new session on next access)
    pub async fn clear_current_session(&self) -> Result<()> {
        let session_file = self.state_dir.join("last_session");
        if session_file.exists() {
            fs::remove_file(session_file)
                .await
                .map_err(|e| OpenCodeError::session_persistence_error(e.to_string()))?;
        }
        Ok(())
    }

    /// Load the last used session ID from local storage
    async fn load_last_session_id(&self) -> Result<String> {
        let session_file = self.state_dir.join("last_session");
        
        if !session_file.exists() {
            return Err(OpenCodeError::session_persistence_error("No saved session"));
        }

        let content = fs::read_to_string(session_file)
            .await
            .map_err(|e| OpenCodeError::session_persistence_error(e.to_string()))?;

        let session_id = content.trim().to_string();
        if session_id.is_empty() {
            return Err(OpenCodeError::session_persistence_error("Empty session ID"));
        }

        Ok(session_id)
    }

    /// Save the session ID to local storage
    async fn save_last_session_id(&self, session_id: &str) -> Result<()> {
        // Ensure state directory exists
        fs::create_dir_all(&self.state_dir)
            .await
            .map_err(|e| OpenCodeError::session_persistence_error(e.to_string()))?;

        let session_file = self.state_dir.join("last_session");
        fs::write(session_file, session_id)
            .await
            .map_err(|e| OpenCodeError::session_persistence_error(e.to_string()))?;

        Ok(())
    }
}

/// Get the OpenCode state directory path
fn get_opencode_state_dir() -> PathBuf {
    // Try HOME environment variable first (standard on Unix/Linux)
    if let Ok(home) = env::var("HOME") {
        PathBuf::from(home).join(".opencode")
    } else {
        // Fallback to current directory if HOME is not available
        PathBuf::from(".opencode")
    }
}

/// High-level session management functions
impl OpenCodeClient {
    /// Get or create a session using session manager
    pub async fn get_or_create_session(&self) -> Result<Session> {
        let manager = SessionManager::new(self.clone());
        manager.get_or_create_session().await
    }

    /// Create a new session and set it as current
    pub async fn create_new_session(&self) -> Result<Session> {
        let manager = SessionManager::new(self.clone());
        manager.create_new_session().await
    }

    /// Get the current session ID if one exists and is valid
    pub async fn get_current_session_id(&self) -> Option<String> {
        let manager = SessionManager::new(self.clone());
        manager.get_current_session_id().await
    }

    /// Switch to a specific session
    pub async fn switch_to_session(&self, session_id: &str) -> Result<Session> {
        let manager = SessionManager::new(self.clone());
        manager.switch_to_session(session_id).await
    }

    /// Clear the current session
    pub async fn clear_current_session(&self) -> Result<()> {
        let manager = SessionManager::new(self.clone());
        manager.clear_current_session().await
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_session_manager() -> (SessionManager, TempDir) {
        let temp_dir = tempfile::tempdir().unwrap();
        let client = OpenCodeClient::new("http://localhost:8080");
        let mut manager = SessionManager::new(client);
        manager.state_dir = temp_dir.path().to_path_buf();
        (manager, temp_dir)
    }

    #[tokio::test]
    async fn test_save_and_load_session_id() {
        let (manager, _temp_dir) = create_test_session_manager();
        
        let session_id = "test-session-123";
        manager.save_last_session_id(session_id).await.unwrap();
        
        let loaded_id = manager.load_last_session_id().await.unwrap();
        assert_eq!(loaded_id, session_id);
    }

    #[tokio::test]
    async fn test_clear_current_session() {
        let (manager, _temp_dir) = create_test_session_manager();
        
        let session_id = "test-session-456";
        manager.save_last_session_id(session_id).await.unwrap();
        
        // Verify it exists
        assert!(manager.load_last_session_id().await.is_ok());
        
        // Clear it
        manager.clear_current_session().await.unwrap();
        
        // Verify it's gone
        assert!(manager.load_last_session_id().await.is_err());
    }

    #[test]
    fn test_get_opencode_state_dir() {
        let state_dir = get_opencode_state_dir();
        assert!(state_dir.ends_with(".opencode"));
    }
}