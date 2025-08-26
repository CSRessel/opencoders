//! Test server management for smoke tests

use crate::common::{find_available_port, wait_for_server_ready, TestConfig};
use eyre::{Result, WrapErr};
use std::process::Stdio;
use tempfile::TempDir;
use tokio::process::{Child, Command};

/// Manages a test instance of the opencode server
pub struct TestServer {
    process: Child,
    base_url: String,
    port: u16,
    _temp_dir: TempDir, // Keep temp dir alive for the duration of the test
}

impl TestServer {
    /// Start a new test server instance
    pub async fn start() -> Result<Self> {
        Self::start_with_config(TestConfig::default()).await
    }

    /// Start a new test server instance with custom configuration
    pub async fn start_with_config(config: TestConfig) -> Result<Self> {
        // Create a temporary directory for the test
        let temp_dir = tempfile::tempdir().wrap_err("Failed to create temporary directory")?;
        
        // Create a main.rs file with dummy code in the temp directory
        let main_rs_path = temp_dir.path().join("main.rs");
        let main_rs_content = r#"fn main() {
    println!("Hello from test server!");
}

#[cfg(test)]
mod tests {
    #[test]
    fn dummy_test() {
        assert_eq!(2 + 2, 4);
    }
}
"#;
        std::fs::write(&main_rs_path, main_rs_content)
            .wrap_err("Failed to create main.rs in temp directory")?;

        // Find an available port
        let port = find_available_port()
            .await
            .wrap_err("Failed to find available port")?;
        // let port = 8080;

        println!(
            "Starting test server on port {} in directory {:?}",
            port,
            temp_dir.path()
        );

        // Start the opencode server process
        let mut process = Command::new("opencode")
            .args(&[
                "serve",
                "--port", &port.to_string(),
                "--hostname", "127.0.0.1",
            ])
            .current_dir(temp_dir.path())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .wrap_err("Failed to start opencode server. Make sure 'opencode' is installed and available in PATH")?;

        let base_url = format!("http://127.0.0.1:{}", port);

        // Wait for the server to be ready
        match wait_for_server_ready(port, config.server_timeout).await {
            Ok(()) => {
                println!("Test server ready at {}", base_url);
                Ok(Self {
                    process,
                    base_url,
                    port,
                    _temp_dir: temp_dir,
                })
            }
            Err(e) => {
                // Kill the process if server failed to start
                let _ = process.kill().await;
                Err(e).wrap_err("Server failed to start within timeout")
            }
        }
    }

    /// Get the base URL of the test server
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Get the port the server is running on
    pub fn port(&self) -> u16 {
        self.port
    }

    /// Check if the server process is still running
    pub fn is_running(&mut self) -> bool {
        match self.process.try_wait() {
            Ok(Some(_)) => false, // Process has exited
            Ok(None) => true,     // Process is still running
            Err(_) => false,      // Error checking status, assume not running
        }
    }

    /// Gracefully shutdown the server
    pub async fn shutdown(mut self) -> Result<()> {
        println!("Shutting down test server on port {}", self.port);

        // Try to terminate gracefully first
        if let Err(e) = self.process.kill().await {
            eprintln!("Warning: Failed to kill server process: {}", e);
        }

        // Wait for the process to exit
        match self.process.wait().await {
            Ok(status) => {
                if status.success() {
                    println!("Server shut down successfully");
                } else {
                    println!("Server exited with status: {}", status);
                }
            }
            Err(e) => {
                eprintln!("Warning: Error waiting for server to exit: {}", e);
            }
        }

        Ok(())
    }
}

impl Drop for TestServer {
    fn drop(&mut self) {
        // Ensure the process is killed when the TestServer is dropped
        if self.is_running() {
            println!(
                "Force killing server process on port {} during cleanup",
                self.port
            );
            let _ = self.process.start_kill();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_server_lifecycle() {
        // This test verifies that we can start and stop a test server
        let server = TestServer::start()
            .await
            .expect("Failed to start test server");

        // Verify server is accessible
        let client = reqwest::Client::new();
        let response = client
            .get(&format!("{}/app", server.base_url()))
            .send()
            .await;
        assert!(response.is_ok(), "Server should be accessible");

        // Shutdown server
        server.shutdown().await.expect("Failed to shutdown server");
    }
}
