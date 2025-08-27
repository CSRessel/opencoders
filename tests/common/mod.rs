//! Common test utilities for smoke tests

pub mod assertions;
pub mod server;
pub use assertions::*;
pub use server::TestServer;
use std::time::Duration;

/// Configuration for test execution
pub struct TestConfig {
    pub server_timeout: Duration,
    pub cleanup_on_failure: bool,
    pub program_path: Option<String>,
    pub program_contents: Option<String>,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            server_timeout: Duration::from_secs(30),
            cleanup_on_failure: true,
            program_path: None,
            program_contents: None,
        }
    }
}

/// Find an available port for testing
pub async fn find_available_port() -> eyre::Result<u16> {
    use tokio::net::TcpListener;
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let port = listener.local_addr()?.port();
    Ok(port)
}

/// Wait for a server to be ready by polling its health endpoint
pub async fn wait_for_server_ready(port: u16, timeout: Duration) -> eyre::Result<()> {
    use tokio::time::{sleep, timeout as tokio_timeout};

    let client = reqwest::Client::new();
    let url = format!("http://localhost:{}/app", port); // Use /app endpoint as health check

    tokio_timeout(timeout, async {
        loop {
            match client.get(&url).send().await {
                Ok(response) if response.status().is_success() => break,
                Ok(response) => {
                    println!("Server not ready yet, status: {}", response.status());
                    sleep(Duration::from_millis(500)).await;
                }
                Err(e) => {
                    println!("Waiting for server to start: {}", e);
                    sleep(Duration::from_millis(500)).await;
                }
            }
        }
    })
    .await
    .map_err(|_| eyre::eyre!("Timeout waiting for server to be ready"))?;

    Ok(())
}
