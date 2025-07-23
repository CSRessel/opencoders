# OpenCode SDK Wrapper

A high-level, ergonomic Rust client for the OpenCode API that wraps the auto-generated OpenAPI client with additional functionality and better error handling.

## Features

- **Type-safe API interactions** with strongly-typed parameters and responses
- **High-level abstractions** over the auto-generated SDK
- **Comprehensive error handling** with specific error types
- **Message builder pattern** for complex message construction
- **Real-time event streaming** with async support
- **Session management** with initialization, sharing, and cleanup
- **File operations** with search and content retrieval
- **Logging integration** with structured log levels

## Quick Start

### Basic Setup

```rust
use opencoders::sdk::{OpenCodeClient, Result};

#[tokio::main]
async fn main() -> Result<()> {
    // Create a new client
    let client = OpenCodeClient::new("http://localhost:8080");
    
    // Get application info
    let app_info = client.get_app_info().await?;
    println!("App: {}", app_info.name);
    
    Ok(())
}
```

### Custom HTTP Client

```rust
use opencoders::sdk::OpenCodeClient;
use reqwest::Client;
use std::time::Duration;

let http_client = Client::builder()
    .timeout(Duration::from_secs(30))
    .build()?;

let client = OpenCodeClient::with_client("http://localhost:8080", http_client);
```

## Session Management

### Creating and Managing Sessions

```rust
use opencoders::sdk::{OpenCodeClient, Result};

async fn session_example() -> Result<()> {
    let client = OpenCodeClient::new("http://localhost:8080");
    
    // Create a new session
    let session = client.create_session().await?;
    println!("Created session: {}", session.id);
    
    // Initialize the session (analyzes app and creates AGENTS.md)
    client.initialize_session(
        &session.id,
        "msg-123",      // message_id
        "anthropic",    // provider_id
        "claude-3-5-sonnet-20241022", // model_id
    ).await?;
    
    // List all sessions
    let sessions = client.list_sessions().await?;
    println!("Total sessions: {}", sessions.len());
    
    // Share a session
    let shared_session = client.share_session(&session.id).await?;
    
    // Clean up
    client.delete_session(&session.id).await?;
    
    Ok(())
}
```

## Sending Messages

### Simple Text Message

```rust
use opencoders::sdk::{OpenCodeClient, LogLevel, Result};

async fn send_text_message() -> Result<()> {
    let client = OpenCodeClient::new("http://localhost:8080");
    let session = client.create_session().await?;
    
    // Build and send a text message
    let response = client
        .message_builder(&session.id)
        .message_id("msg-456")
        .provider("anthropic")
        .model("claude-3-5-sonnet-20241022")
        .mode("chat")
        .add_text_part("Hello, can you help me with my Rust code?")
        .send(&client.config)
        .await?;
    
    println!("Assistant response: {}", response.id);
    Ok(())
}
```

### Message with File Attachment

```rust
async fn send_message_with_file() -> Result<()> {
    let client = OpenCodeClient::new("http://localhost:8080");
    let session = client.create_session().await?;
    
    let response = client
        .message_builder(&session.id)
        .message_id("msg-789")
        .provider("anthropic")
        .model("claude-3-5-sonnet-20241022")
        .mode("chat")
        .add_text_part("Please review this code:")
        .add_file_part("main.rs", "application/rust", "/path/to/main.rs")
        .send(&client.config)
        .await?;
    
    println!("Message sent with file attachment");
    Ok(())
}
```

### Retrieving Messages

```rust
async fn get_session_messages() -> Result<()> {
    let client = OpenCodeClient::new("http://localhost:8080");
    
    let messages = client.get_messages("session-id-123").await?;
    
    for message in messages {
        println!("Message ID: {}", message.id);
        // Handle different message types
    }
    
    Ok(())
}
```

## File Operations

### Reading Files

```rust
async fn file_operations() -> Result<()> {
    let client = OpenCodeClient::new("http://localhost:8080");
    
    // Read a specific file
    let file_content = client.read_file("/path/to/file.rs").await?;
    println!("File content: {}", file_content.content);
    
    // Get file status for all files
    let file_statuses = client.get_file_status().await?;
    for file in file_statuses {
        println!("File: {} ({})", file.path, file.status);
    }
    
    Ok(())
}
```

### Search Operations

```rust
async fn search_operations() -> Result<()> {
    let client = OpenCodeClient::new("http://localhost:8080");
    
    // Search for text in files
    let matches = client.find_text("fn main").await?;
    for match_result in matches {
        println!("Found in: {} at line {}", match_result.path, match_result.line);
    }
    
    // Find files by name pattern
    let files = client.find_files("*.rs").await?;
    for file in files {
        println!("Rust file: {}", file);
    }
    
    // Find symbols (functions, structs, etc.)
    let symbols = client.find_symbols("OpenCodeClient").await?;
    for symbol in symbols {
        println!("Symbol: {} in {}", symbol.name, symbol.location.path);
    }
    
    Ok(())
}
```

## Configuration and Providers

### Getting Configuration

```rust
async fn configuration_example() -> Result<()> {
    let client = OpenCodeClient::new("http://localhost:8080");
    
    // Get current configuration
    let config = client.get_config().await?;
    println!("Config version: {}", config.version);
    
    // Get available providers
    let providers = client.get_providers().await?;
    for provider in providers.providers {
        println!("Provider: {} ({})", provider.id, provider.name);
    }
    
    // Get available modes
    let modes = client.get_modes().await?;
    for mode in modes {
        println!("Mode: {} - {}", mode.id, mode.description);
    }
    
    Ok(())
}
```

## Logging

### Writing Log Entries

```rust
use opencoders::sdk::{OpenCodeClient, LogLevel, Result};
use std::collections::HashMap;

async fn logging_example() -> Result<()> {
    let client = OpenCodeClient::new("http://localhost:8080");
    
    // Simple log entry
    client.write_log(
        "my-service",
        LogLevel::Info,
        "Application started successfully",
        None,
    ).await?;
    
    // Log with extra metadata
    let mut extra = HashMap::new();
    extra.insert("user_id".to_string(), serde_json::Value::String("user-123".to_string()));
    extra.insert("duration_ms".to_string(), serde_json::Value::Number(serde_json::Number::from(150)));
    
    client.write_log(
        "auth-service",
        LogLevel::Debug,
        "User authentication completed",
        Some(extra),
    ).await?;
    
    Ok(())
}
```

## Real-time Events

### Event Streaming

```rust
use opencoders::sdk::{OpenCodeClient, Result};

async fn event_streaming_example() -> Result<()> {
    let mut client = OpenCodeClient::new("http://localhost:8080");
    
    // Subscribe to real-time events
    let mut event_handle = client.subscribe_to_events().await?;
    
    // Listen for events
    tokio::spawn(async move {
        while let Some(event) = event_handle.next_event().await {
            match event.event_type.as_str() {
                "session.updated" => {
                    println!("Session updated: {:?}", event.properties);
                }
                "message.updated" => {
                    println!("Message updated: {:?}", event.properties);
                }
                "file.edited" => {
                    println!("File edited: {:?}", event.properties);
                }
                _ => {
                    println!("Unknown event: {}", event.event_type);
                }
            }
        }
    });
    
    // Your application logic here...
    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
    
    Ok(())
}
```

### Non-blocking Event Polling

```rust
async fn poll_events_example() -> Result<()> {
    let mut client = OpenCodeClient::new("http://localhost:8080");
    let mut event_handle = client.subscribe_to_events().await?;
    
    loop {
        // Try to get an event without blocking
        if let Some(event) = event_handle.try_next_event() {
            println!("Received event: {}", event.event_type);
        }
        
        // Do other work...
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        // Check if event stream is still active
        if !event_handle.is_active() {
            println!("Event stream closed");
            break;
        }
    }
    
    Ok(())
}
```

## Error Handling

### Comprehensive Error Handling

```rust
use opencoders::sdk::{OpenCodeClient, OpenCodeError, Result};

async fn error_handling_example() -> Result<()> {
    let client = OpenCodeClient::new("http://localhost:8080");
    
    match client.get_app_info().await {
        Ok(app) => println!("App: {}", app.name),
        Err(OpenCodeError::Http(e)) => {
            eprintln!("HTTP error: {}", e);
            // Handle network issues
        }
        Err(OpenCodeError::Api { status, message }) => {
            eprintln!("API error {}: {}", status, message);
            // Handle API-specific errors
        }
        Err(OpenCodeError::Serialization(e)) => {
            eprintln!("Serialization error: {}", e);
            // Handle JSON parsing issues
        }
        Err(e) => {
            eprintln!("Other error: {}", e);
            
            // Check error properties
            if e.is_retryable() {
                println!("This error can be retried");
            }
            if e.is_client_error() {
                println!("This is a client error (4xx)");
            }
            if e.is_server_error() {
                println!("This is a server error (5xx)");
            }
        }
    }
    
    Ok(())
}
```

## Advanced Usage

### Session Summarization

```rust
async fn summarize_session_example() -> Result<()> {
    let client = OpenCodeClient::new("http://localhost:8080");
    
    // Summarize a session using AI
    client.summarize_session(
        "session-id-123",
        "anthropic",
        "claude-3-5-sonnet-20241022",
    ).await?;
    
    println!("Session summary generated");
    Ok(())
}
```

### Session Control

```rust
async fn session_control_example() -> Result<()> {
    let client = OpenCodeClient::new("http://localhost:8080");
    let session_id = "session-id-123";
    
    // Abort a running session
    client.abort_session(session_id).await?;
    println!("Session aborted");
    
    // Share a session
    let shared_session = client.share_session(session_id).await?;
    println!("Session shared with ID: {}", shared_session.share_id);
    
    // Unshare a session
    let unshared_session = client.unshare_session(session_id).await?;
    println!("Session unshared");
    
    Ok(())
}
```

## Type Aliases

The SDK provides convenient type aliases for common identifiers:

```rust
use opencoders::sdk::{SessionId, MessageId, ProviderId, ModelId};

let session_id: SessionId = "sess-123".to_string();
let message_id: MessageId = "msg-456".to_string();
let provider_id: ProviderId = "anthropic".to_string();
let model_id: ModelId = "claude-3-5-sonnet-20241022".to_string();
```

## Best Practices

1. **Reuse the client**: Create one `OpenCodeClient` instance and reuse it across your application
2. **Handle errors appropriately**: Use the error classification methods to determine retry strategies
3. **Use message builders**: For complex messages, prefer the builder pattern over manual construction
4. **Resource cleanup**: Always delete sessions when done to free up resources
5. **Event streaming**: Use event streaming for real-time updates rather than polling APIs
6. **Structured logging**: Include relevant metadata in log entries for better observability

## Dependencies

Add these dependencies to your `Cargo.toml`:

```toml
[dependencies]
opencoders = { path = "path/to/opencoders" }
tokio = { version = "1.0", features = ["full"] }
serde_json = "1.0"
uuid = { version = "1.0", features = ["v4"] }
```

## Thread Safety

The `OpenCodeClient` is `Send + Sync` and can be safely shared across threads:

```rust
use std::sync::Arc;

let client = Arc::new(OpenCodeClient::new("http://localhost:8080"));

// Clone for use in async tasks
let client_clone = client.clone();
tokio::spawn(async move {
    let app_info = client_clone.get_app_info().await.unwrap();
    println!("App: {}", app_info.name);
});
```

## Type Derive Conventions

The SDK follows Rust conventions for trait derivations to provide ergonomic APIs:

### Standard Derives

All public types implement these standard traits where possible:

- **`Debug`**: All public types implement `Debug` for debugging and logging
- **`Clone`**: Most types implement `Clone` for easy duplication and sharing
- **`PartialEq`**: Value types implement `PartialEq` for comparison

### Error Type Handling

The `OpenCodeError` type uses a hybrid approach for derives:

```rust
#[derive(Error, Debug)]
pub enum OpenCodeError {
    // HTTP and serialization errors use custom Clone/PartialEq implementations
    // because the underlying error types don't support these traits
    Http(#[from] reqwest::Error),
    Serialization(#[from] serde_json::Error),
    
    // All other variants can be compared and cloned normally
    Api { status: u16, message: String },
    // ... other variants
}
```

**Custom implementations preserve error information:**
- `Clone`: Converts non-cloneable errors to `Unexpected` with preserved error messages
- `PartialEq`: Compares errors by their string representation when direct comparison isn't possible

### Builder Pattern Types

Builder types like `MessageBuilder` implement:
- `#[derive(Debug, Clone)]` for debugging and method chaining
- Methods consume `self` and return `Self` for fluent interfaces

### Event Stream Types

Event streaming types have specialized derive implementations:
- `EventStream`: `#[derive(Debug)]` only (contains non-cloneable async handles)
- `EventStreamHandle`: `#[derive(Debug)]` + custom `Clone` using `resubscribe()`

### Configuration Types

Configuration structs use comprehensive derives:
```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveryConfig {
    // ... fields
}
```

This provides maximum ergonomics for configuration management and testing.