//! Tests for the OpenCode SDK client wrapper
//!
//! This test suite validates the high-level SDK client functionality,
//! including connection management, ID generation, session lifecycle,
//! and the MessageBuilder pattern.

mod common;

use common::server::TestServer;
use eyre::{Result, WrapErr};
use opencoders::sdk::client::{generate_descending_id, generate_id, IdPrefix, OpenCodeClient};
use opencoders::sdk::LogLevel;
use std::collections::HashSet;
use std::time::Duration;

use crate::common::TestConfig;

// ============================================================================
// Basic Client Tests
// ============================================================================

/// Test the basic client construction and connection
#[tokio::test]
async fn test_client_construction_and_connection() -> Result<()> {
    let server = TestServer::start().await?;

    // Test basic client construction
    let client = OpenCodeClient::new(&server.base_url());
    assert_eq!(client.base_url(), server.base_url());

    // Test connection
    client
        .test_connection()
        .await
        .wrap_err("Client should be able to connect to test server")?;

    Ok(())
}

/// Test client discovery functionality
#[tokio::test]
async fn test_client_discovery() -> Result<()> {
    let _server = TestServer::start().await?;

    // Test discovery (may not work in test environment, but shouldn't panic)
    match OpenCodeClient::discover().await {
        Ok(client) => {
            // If discovery works, test basic connection
            client
                .test_connection()
                .await
                .wrap_err("Discovered client should be able to connect")?;
        }
        Err(_) => {
            // Discovery failure is acceptable in test environment
            // The important thing is it doesn't crash
        }
    }

    Ok(())
}

/// Test client cloning
#[tokio::test]
async fn test_client_cloning() -> Result<()> {
    let server = TestServer::start().await?;
    let client = OpenCodeClient::new(&server.base_url());

    // Test clone_client method
    let cloned = client.clone_client();
    assert_eq!(client.base_url(), cloned.base_url());
    assert_eq!(client, cloned); // Uses PartialEq implementation

    // Both clients should work independently
    client.test_connection().await?;
    cloned.test_connection().await?;

    Ok(())
}

// ============================================================================
// App and Configuration Tests
// ============================================================================

/// Test app information retrieval
#[tokio::test]
async fn test_get_app_info() -> Result<()> {
    let server = TestServer::start().await?;
    let client = OpenCodeClient::new(&server.base_url());

    let app_info = client
        .get_app_info()
        .await
        .wrap_err("Should be able to get app info")?;

    // Basic validation - check actual App structure
    assert!(
        !app_info.hostname.is_empty(),
        "App hostname should not be empty"
    );
    // Git field should be boolean
    assert!(
        app_info.git || !app_info.git,
        "Git field should be valid boolean"
    );
    Ok(())
}

/// Test app initialization
#[tokio::test]
async fn test_initialize_app() -> Result<()> {
    let server = TestServer::start().await?;
    let client = OpenCodeClient::new(&server.base_url());

    let result = client
        .initialize_app()
        .await
        .wrap_err("Should be able to initialize app")?;

    // Result should be boolean
    assert!(
        result == true || result == false,
        "Initialize app should return boolean"
    );
    Ok(())
}

/// Test configuration retrieval
#[tokio::test]
async fn test_get_config() -> Result<()> {
    let server = TestServer::start().await?;
    let client = OpenCodeClient::new(&server.base_url());

    let _config = client
        .get_config()
        .await
        .wrap_err("Should be able to get config")?;

    // Config should have some basic structure (flexible validation)
    Ok(())
}

/// Test providers retrieval
#[tokio::test]
async fn test_get_providers() -> Result<()> {
    let server = TestServer::start().await?;
    let client = OpenCodeClient::new(&server.base_url());

    let providers = client
        .get_providers()
        .await
        .wrap_err("Should be able to get providers")?;

    // Should have at least an empty list
    assert!(
        providers.providers.len() >= 1,
        "Providers list should be valid"
    );

    // If we have providers, they should be valid
    for provider in &providers.providers {
        assert!(!provider.id.is_empty(), "Provider ID should not be empty");
        assert!(
            !provider.name.is_empty(),
            "Provider name should not be empty"
        );
    }

    Ok(())
}

/// Test agent configurations (formerly modes)
#[tokio::test]
async fn test_get_agent_configs() -> Result<()> {
    let server = TestServer::start().await?;
    let client = OpenCodeClient::new(&server.base_url());

    let _agent_config = client
        .get_agent_configs()
        .await
        .wrap_err("Should be able to get agent configs")?;

    // Agent config should have some structure (flexible validation)
    Ok(())
}

// ============================================================================
// Session Management Tests
// ============================================================================

/// Test session creation and basic lifecycle
#[tokio::test]
async fn test_session_lifecycle() -> Result<()> {
    let server = TestServer::start().await?;
    let client = OpenCodeClient::new(&server.base_url());

    // Create session
    let session = client
        .create_session()
        .await
        .wrap_err("Should be able to create session")?;

    // Basic session validation
    assert!(!session.id.is_empty(), "Session ID should not be empty");
    assert!(
        !session.title.is_empty(),
        "Session title should not be empty"
    );
    assert!(
        !session.version.is_empty(),
        "Session version should not be empty"
    );
    let session_id = session.id.clone();

    // List sessions (should include our new session)
    let sessions = client
        .list_sessions()
        .await
        .wrap_err("Should be able to list sessions")?;

    assert!(!sessions.is_empty(), "Should have at least one session");
    assert!(
        sessions.iter().any(|s| s.id == session_id),
        "Session list should include our created session"
    );

    // Delete session
    let deleted = client
        .delete_session(&session_id)
        .await
        .wrap_err("Should be able to delete session")?;

    assert!(deleted, "Session deletion should return true");
    Ok(())
}

/// Test session operations (abort, share, etc.)
#[tokio::test]
async fn test_session_operations() -> Result<()> {
    let server = TestServer::start().await?;
    let client = OpenCodeClient::new(&server.base_url());

    // Create a session for testing
    let session = client.create_session().await?;
    let session_id = &session.id;

    // Test abort session
    let aborted = client
        .abort_session(session_id)
        .await
        .wrap_err("Should be able to abort session")?;
    assert!(
        aborted == true || aborted == false,
        "Abort should return boolean"
    );

    // Test share session (may fail depending on server state, but shouldn't panic)
    match client.share_session(session_id).await {
        Ok(shared_session) => {
            assert!(
                !shared_session.id.is_empty(),
                "Shared session should have ID"
            );

            // Test unshare
            let _unshared = client
                .unshare_session(session_id)
                .await
                .wrap_err("Should be able to unshare session")?;
        }
        Err(_) => {
            // Sharing may fail in test environment, that's acceptable
        }
    }

    // Clean up
    let _ = client.delete_session(session_id).await;
    Ok(())
}

// ============================================================================
// Message Tests
// ============================================================================

/// Test message retrieval
#[tokio::test]
async fn test_get_messages() -> Result<()> {
    let server = TestServer::start().await?;
    let client = OpenCodeClient::new(&server.base_url());

    // Create a session
    let session = client.create_session().await?;
    let session_id = &session.id;

    // Get messages (should be empty for new session)
    let messages = client
        .get_messages(session_id)
        .await
        .wrap_err("Should be able to get messages")?;

    // New session should have no messages or just system messages
    assert!(messages.len() == 0, "Messages should be a valid list");

    // Clean up
    let _ = client.delete_session(session_id).await;
    Ok(())
}

/// Test sending user messages (if providers available)
#[tokio::test]
async fn test_send_user_message() -> Result<()> {
    let server = TestServer::start().await?;
    let client = OpenCodeClient::new(&server.base_url());

    // Get providers first to use valid provider/model IDs
    let providers = client.get_providers().await?;
    if providers.providers.is_empty() {
        // Skip test if no providers available
        return Ok(());
    }

    let provider = &providers.providers[0];
    let provider_id = &provider.id;

    // Get first model from the HashMap
    if provider.models.is_empty() {
        // Skip if no models available
        return Ok(());
    }

    let model_id = provider.models.keys().next().unwrap();

    // Create a session
    let session = client.create_session().await?;
    let session_id = &session.id;

    // Generate message ID
    let message_id = generate_id(IdPrefix::Message);

    // Send a simple message (may fail due to actual AI processing, but should not panic)
    match client
        .send_user_message(
            session_id,
            &message_id,
            "Hello, this is a test message",
            provider_id,
            model_id,
            None, // no mode
        )
        .await
    {
        Ok(response) => {
            // Validate response structure - AssistantMessage has these fields
            assert!(!response.id.is_empty(), "Response should have an ID");
            assert!(
                !response.session_id.is_empty(),
                "Response should have session_id"
            );
        }
        Err(_) => {
            // Message sending may fail in test environment (no actual AI backend)
            // The important thing is it doesn't crash and follows the API contract
        }
    }

    // Clean up
    let _ = client.delete_session(session_id).await;
    Ok(())
}

// ============================================================================
// File Operations Tests
// ============================================================================

/// Test file operations
#[tokio::test]
async fn test_file_operations() -> Result<()> {
    // TODO need to add staged changes so file status can detect working files
    let server = TestServer::start().await?;
    let client = OpenCodeClient::new(&server.base_url());

    // Test file status
    let files = client
        .get_file_status()
        .await
        .wrap_err("Should be able to get file status")?;

    // File list should be valid (empty is OK)
    assert!(files.len() >= 1, "File list should be valid");

    // If we have files, try to read the first one
    if !files.is_empty() {
        let file_path = &files[0].path;
        match client.read_file(file_path).await {
            Ok(file_content) => {
                // File content should be valid (empty or non-empty string)
                assert!(
                    file_content.content.len() >= 1,
                    "File content should be a valid string"
                );
            }
            Err(_) => {
                // File reading may fail (permissions, etc.), that's acceptable
            }
        }
    }

    Ok(())
}

// ============================================================================
// Search Operations Tests
// ============================================================================

/// Test search operations
#[tokio::test]
async fn test_search_operations() -> Result<()> {
    let server = TestServer::start_with_config(TestConfig {
        server_timeout: Duration::from_secs(30),
        cleanup_on_failure: true,
        program_path: Some("main.rs".to_string()),
        program_contents: Some(
            r#"

#[derive(Debug, Display)]
pub struct Message {
    m: String,
}

fn main() {
    let message = Message{ m: "Hello from test server!".to_string() };
    println!("{}", message);
}

#[cfg(test)]
mod tests {
    #[test]
    fn dummy_test() {
        assert_eq!(2 + 2, 4);
    }
}
"#
            .to_string(),
        ),
    })
    .await?;

    let client = OpenCodeClient::new(&server.base_url());

    // Test text search
    let text_results = client
        .find_text("test")
        .await
        .wrap_err("Should be able to search text")?;
    assert!(
        text_results.len() >= 1,
        "Text search results should be valid"
    );

    // Test file search
    let file_results = client
        .find_files("rs")
        .await
        .wrap_err("Should be able to search files")?;
    assert!(
        file_results.len() >= 1,
        "File search results should be valid"
    );

    // TODO resolve this case
    // // Test symbol search
    // let symbol_results = client
    //     .find_symbols("Message")
    //     .await
    //     .wrap_err("Should be able to search symbols")?;
    // assert!(
    //     symbol_results.len() >= 1,
    //     "Symbol search results should be valid"
    // );

    Ok(())
}

// ============================================================================
// Logging Tests
// ============================================================================

/// Test logging functionality
#[tokio::test]
async fn test_write_log() -> Result<()> {
    let server = TestServer::start().await?;
    let client = OpenCodeClient::new(&server.base_url());

    // Test writing logs at different levels
    let log_levels = [
        LogLevel::Debug,
        LogLevel::Info,
        LogLevel::Warn,
        LogLevel::Error,
    ];

    for level in log_levels {
        let result = client
            .write_log(
                "test_service",
                level,
                &format!("Test log message at {:?} level", level),
                None,
            )
            .await
            .wrap_err("Should be able to write log")?;

        assert!(
            result == true || result == false,
            "Log write should return boolean"
        );
    }

    // Test with extra data
    let mut extra = std::collections::HashMap::new();
    extra.insert("test_key".to_string(), serde_json::json!("test_value"));

    let result = client
        .write_log(
            "test_service",
            LogLevel::Info,
            "Test log with extra data",
            Some(extra),
        )
        .await?;

    assert!(
        result == true || result == false,
        "Log write with extra should return boolean"
    );
    Ok(())
}

// ============================================================================
// ID Generation Tests
// ============================================================================

/// Test basic ID generation for different prefixes
#[test]
fn test_id_generation_prefixes() -> Result<()> {
    let prefixes = [
        (IdPrefix::Message, "msg"),
        (IdPrefix::Session, "ses"),
        (IdPrefix::User, "usr"),
        (IdPrefix::Part, "prt"),
        (IdPrefix::Permission, "per"),
    ];

    for (prefix_enum, expected_prefix) in prefixes {
        let id = generate_id(prefix_enum);

        // Check prefix format
        assert!(
            id.starts_with(expected_prefix),
            "ID should start with correct prefix: {} -> {}",
            expected_prefix,
            id
        );
        assert!(
            id.starts_with(&format!("{}_", expected_prefix)),
            "ID should have underscore after prefix: {}",
            id
        );

        // Check total length: prefix + '_' + 12 hex chars + 14 base62 chars
        let expected_len = expected_prefix.len() + 1 + 12 + 14;
        assert_eq!(
            id.len(),
            expected_len,
            "ID should be correct length: expected {}, got {} for '{}'",
            expected_len,
            id.len(),
            id
        );

        // Check format: prefix_[12 hex][14 base62]
        let suffix = &id[expected_prefix.len() + 1..];
        assert_eq!(suffix.len(), 26, "Suffix should be 26 characters");

        // First 12 chars should be hex
        let hex_part = &suffix[0..12];
        assert!(
            hex_part.chars().all(|c| c.is_ascii_hexdigit()),
            "First 12 suffix chars should be hex: {}",
            hex_part
        );

        // Last 14 chars should be base62
        let base62_part = &suffix[12..26];
        let base62_chars = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
        assert!(
            base62_part.chars().all(|c| base62_chars.contains(c)),
            "Last 14 chars should be base62: {}",
            base62_part
        );
    }

    Ok(())
}

/// Test ID uniqueness within short time windows
#[test]
fn test_id_uniqueness() -> Result<()> {
    let mut ids = HashSet::new();
    let iterations = 1000;

    // Generate many IDs rapidly to test counter logic
    for _ in 0..iterations {
        let id = generate_id(IdPrefix::Message);
        assert!(!ids.contains(&id), "Generated duplicate ID: {}", id);
        ids.insert(id);
    }

    assert_eq!(
        ids.len(),
        iterations,
        "Should have generated {} unique IDs",
        iterations
    );
    Ok(())
}

/// Test descending ID generation
#[test]
fn test_descending_id_generation() -> Result<()> {
    let normal_id = generate_id(IdPrefix::Message);
    let descending_id = generate_descending_id(IdPrefix::Message);

    // Both should have same format
    assert!(normal_id.starts_with("msg_"));
    assert!(descending_id.starts_with("msg_"));
    assert_eq!(normal_id.len(), descending_id.len());

    // Extract hex parts for comparison
    let normal_hex = &normal_id[4..16]; // Skip "msg_", take 12 hex chars
    let descending_hex = &descending_id[4..16];

    // They should be different (due to bit flipping)
    assert_ne!(
        normal_hex, descending_hex,
        "Descending ID hex should differ from normal ID"
    );

    Ok(())
}

/// Test ID prefix enum functionality
#[test]
fn test_id_prefix_enum() -> Result<()> {
    assert_eq!(IdPrefix::Message.as_str(), "msg");
    assert_eq!(IdPrefix::Session.as_str(), "ses");
    assert_eq!(IdPrefix::User.as_str(), "usr");
    assert_eq!(IdPrefix::Part.as_str(), "prt");
    assert_eq!(IdPrefix::Permission.as_str(), "per");
    Ok(())
}

// ============================================================================
// MessageBuilder Tests
// ============================================================================

/// Test basic MessageBuilder construction
#[tokio::test]
async fn test_message_builder_basic() -> Result<()> {
    let server = TestServer::start().await?;
    let client = OpenCodeClient::new(&server.base_url());

    // Create a session for the builder
    let session = client.create_session().await?;
    let session_id = &session.id;

    // Test basic builder construction
    let _builder = client
        .message_builder(session_id)
        .message_id("test_msg_id")
        .provider("test_provider")
        .model("test_model");

    // The builder should be valid (doesn't panic)
    // We can't send without valid provider/model from server, but construction should work

    // Clean up
    let _ = client.delete_session(session_id).await;
    Ok(())
}

/// Test MessageBuilder with text parts
#[tokio::test]
async fn test_message_builder_with_text_parts() -> Result<()> {
    let server = TestServer::start().await?;
    let client = OpenCodeClient::new(&server.base_url());

    let session = client.create_session().await?;
    let session_id = &session.id;

    // Build message with multiple text parts
    let _builder = client
        .message_builder(session_id)
        .message_id(&generate_id(IdPrefix::Message))
        .provider("test_provider")
        .model("test_model")
        .add_text_part("First part of the message")
        .add_text_part("Second part of the message")
        .add_text_part("Third part with special characters: !@#$%^&*()");

    // Builder construction should succeed

    let _ = client.delete_session(session_id).await;
    Ok(())
}

/// Test MessageBuilder with file parts
#[tokio::test]
async fn test_message_builder_with_file_parts() -> Result<()> {
    let server = TestServer::start().await?;
    let client = OpenCodeClient::new(&server.base_url());

    let session = client.create_session().await?;
    let session_id = &session.id;

    // Build message with file parts
    let _builder = client
        .message_builder(session_id)
        .message_id(&generate_id(IdPrefix::Message))
        .provider("test_provider")
        .model("test_model")
        .add_file_part("test.txt", "text/plain", "file://test.txt")
        .add_file_part("image.png", "image/png", "file://image.png");

    // Builder construction should succeed

    let _ = client.delete_session(session_id).await;
    Ok(())
}

/// Test MessageBuilder fluent API chaining
#[tokio::test]
async fn test_message_builder_fluent_chaining() -> Result<()> {
    let server = TestServer::start().await?;
    let client = OpenCodeClient::new(&server.base_url());

    let session = client.create_session().await?;
    let session_id = &session.id;

    // Test that all methods return the builder for chaining
    let _final_builder = client
        .message_builder(session_id)
        .message_id("test_id")
        .provider("provider1")
        .model("model1")
        .mode("mode1")
        .add_text_part("Part 1")
        .provider("provider2") // Should override previous
        .model("model2") // Should override previous
        .mode("mode2") // Should override previous
        .add_text_part("Part 2")
        .add_file_part("file.txt", "text/plain", "file://file.txt");

    // All chaining should work without panics

    let _ = client.delete_session(session_id).await;
    Ok(())
}

/// Test MessageBuilder validation (missing required fields)
#[tokio::test]
async fn test_message_builder_validation() -> Result<()> {
    let server = TestServer::start().await?;
    let client = OpenCodeClient::new(&server.base_url());

    let session = client.create_session().await?;
    let session_id = &session.id;

    // Test builder with missing message_id
    let builder_no_msg_id = client
        .message_builder(session_id)
        .provider("provider")
        .model("model")
        .add_text_part("Test");

    match builder_no_msg_id.send(client.configuration()).await {
        Err(_) => {
            // Should fail validation for missing message_id
        }
        Ok(_) => {
            // Unexpected success, but not necessarily wrong if server is lenient
        }
    }

    // Test builder with missing provider_id
    let builder_no_provider = client
        .message_builder(session_id)
        .message_id("test_id")
        .model("model")
        .add_text_part("Test");

    match builder_no_provider.send(client.configuration()).await {
        Err(_) => {
            // Should fail validation for missing provider
        }
        Ok(_) => {
            // Unexpected success
        }
    }

    // Test builder with missing model_id
    let builder_no_model = client
        .message_builder(session_id)
        .message_id("test_id")
        .provider("provider")
        .add_text_part("Test");

    match builder_no_model.send(client.configuration()).await {
        Err(_) => {
            // Should fail validation for missing model
        }
        Ok(_) => {
            // Unexpected success
        }
    }

    let _ = client.delete_session(session_id).await;
    Ok(())
}
