//! OpenCode server discovery utilities
//!
//! This module provides functionality to discover and connect to running
//! OpenCode server instances through various methods.

use crate::sdk::{error::{OpenCodeError, Result}, OpenCodeClient};
use std::time::Duration;
use tokio::process::Command;

/// Configuration for server discovery
#[derive(Debug, Clone)]
pub struct DiscoveryConfig {
    /// Timeout for server validation requests
    pub validation_timeout: Duration,
    /// Maximum number of retry attempts
    pub max_retries: u32,
    /// Base delay between retries (exponential backoff)
    pub retry_delay: Duration,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            validation_timeout: Duration::from_secs(5),
            max_retries: 3,
            retry_delay: Duration::from_millis(500),
        }
    }
}

/// Discover a running OpenCode server instance
pub async fn discover_opencode_server() -> Result<String> {
    discover_opencode_server_with_config(DiscoveryConfig::default()).await
}

/// Discover a running OpenCode server instance with custom configuration
pub async fn discover_opencode_server_with_config(config: DiscoveryConfig) -> Result<String> {
    // 1. Check environment variable
    if let Ok(url) = std::env::var("OPENCODE_SERVER_URL") {
        if validate_server_with_config(&url, &config).await.is_ok() {
            return Ok(url);
        }
    }

    // 2. Process detection (platform-specific)
    if let Ok(url) = detect_running_process().await {
        if validate_server_with_config(&url, &config).await.is_ok() {
            return Ok(url);
        }
    }

    Err(OpenCodeError::ServerNotFound)
}

/// Validate that a server is running and accessible at the given URL
pub async fn validate_server(url: &str) -> Result<()> {
    validate_server_with_config(url, &DiscoveryConfig::default()).await
}

/// Validate server with custom configuration
pub async fn validate_server_with_config(url: &str, config: &DiscoveryConfig) -> Result<()> {
    let client = OpenCodeClient::new(url);
    
    for attempt in 0..config.max_retries {
        match tokio::time::timeout(config.validation_timeout, client.get_app_info()).await {
            Ok(Ok(_)) => return Ok(()),
            Ok(Err(e)) => {
                if attempt == config.max_retries - 1 {
                    return Err(e);
                }
            }
            Err(_) => {
                if attempt == config.max_retries - 1 {
                    return Err(OpenCodeError::ConnectionTimeout);
                }
            }
        }
        
        // Exponential backoff
        let delay = config.retry_delay * (2_u32.pow(attempt));
        tokio::time::sleep(delay).await;
    }
    
    Err(OpenCodeError::ConnectionTimeout)
}

/// Detect running OpenCode processes and extract server URLs
async fn detect_running_process() -> Result<String> {
    // Try to find opencode serve processes
    let output = Command::new("ps")
        .args(&["aux"])
        .output()
        .await
        .map_err(|_| OpenCodeError::ProcessDetectionFailed)?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Look for opencode serve processes
    for line in stdout.lines() {
        if line.contains("opencode") && line.contains("serve") {
            // Extract port from command line arguments
            if let Some(url) = extract_server_url_from_process_line(line) {
                return Ok(url);
            }
        }
    }

    Err(OpenCodeError::ProcessDetectionFailed)
}

/// Extract server URL from a process command line
fn extract_server_url_from_process_line(line: &str) -> Option<String> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    
    let mut hostname = "127.0.0.1";
    let mut port = None;
    
    // Look for --port and --hostname arguments
    for i in 0..parts.len() {
        match parts[i] {
            "--port" | "-p" => {
                if i + 1 < parts.len() {
                    port = parts[i + 1].parse::<u16>().ok();
                }
            }
            "--hostname" | "-h" => {
                if i + 1 < parts.len() {
                    hostname = parts[i + 1];
                }
            }
            _ => {}
        }
    }
    
    port.map(|p| format!("http://{}:{}", hostname, p))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_server_url_from_process_line() {
        let line = "user  12345  0.1  0.5  123456  7890 ?  S  10:30  0:01 opencode serve --port 8080 --hostname 127.0.0.1";
        let url = extract_server_url_from_process_line(line);
        assert_eq!(url, Some("http://127.0.0.1:8080".to_string()));
        
        let line2 = "user  12346  0.1  0.5  123456  7890 ?  S  10:30  0:01 opencode serve -p 3000";
        let url2 = extract_server_url_from_process_line(line2);
        assert_eq!(url2, Some("http://127.0.0.1:3000".to_string()));
        
        let line3 = "user  12347  0.1  0.5  123456  7890 ?  S  10:30  0:01 opencode serve --hostname localhost --port 8000";
        let url3 = extract_server_url_from_process_line(line3);
        assert_eq!(url3, Some("http://localhost:8000".to_string()));
    }
}