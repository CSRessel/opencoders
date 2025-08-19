//! Event stream handling for real-time updates

use crate::sdk::error::{OpenCodeError, Result};
use opencode_sdk::{apis::configuration::Configuration, models::Event};
use std::time::Duration;
use tokio::sync::broadcast;

/// Event stream for receiving real-time updates from the OpenCode server
#[derive(Debug)]
pub struct EventStream {
    sender: broadcast::Sender<Event>,
    _handle: tokio::task::JoinHandle<()>,
}

impl EventStream {
    /// Create a new event stream
    pub async fn new(config: Configuration) -> Result<Self> {
        let (sender, _) = broadcast::channel(1000);

        let sender_clone = sender.clone();
        let config_clone = config.clone();

        // Start the polling task
        let handle = tokio::spawn(async move {
            Self::poll_events(config_clone, sender_clone).await;
        });

        Ok(Self {
            sender,
            _handle: handle,
        })
    }

    /// Get a handle to subscribe to events
    pub fn handle(&self) -> EventStreamHandle {
        EventStreamHandle {
            receiver: self.sender.subscribe(),
        }
    }

    /// Internal SSE stream processing for events
    async fn poll_events(config: Configuration, sender: broadcast::Sender<Event>) {
        let mut consecutive_errors = 0;
        const MAX_CONSECUTIVE_ERRORS: u32 = 10;
        
        loop {
            tracing::debug!("Starting SSE stream connection to /event");
            
            match Self::connect_sse_stream(&config).await {
                Ok(()) => {
                    consecutive_errors = 0;
                    tracing::info!("SSE stream connected successfully");
                    
                    // Process the SSE stream
                    if let Err(e) = Self::process_sse_stream(&config, &sender).await {
                        tracing::warn!("SSE stream processing error: {}", e);
                        consecutive_errors += 1;
                    }
                }
                Err(e) => {
                    consecutive_errors += 1;
                    tracing::error!("SSE connection error ({}): {}", consecutive_errors, e);

                    if consecutive_errors >= MAX_CONSECUTIVE_ERRORS {
                        tracing::error!("Too many consecutive errors, stopping event stream");
                        break;
                    }
                }
            }

            if consecutive_errors > 0 {
                // Exponential backoff on errors
                let backoff_duration =
                    Duration::from_millis(1000 * (2_u64.pow(consecutive_errors.min(6))));
                tracing::debug!("Backing off for {:?} before retry", backoff_duration);
                tokio::time::sleep(backoff_duration).await;
            }
        }
    }
    
    /// Connect to SSE stream and verify connection
    async fn connect_sse_stream(config: &Configuration) -> Result<()> {
        let event_url = format!("{}/event", config.base_path);
        let client = &config.client;
        
        // Test connection first
        let response = client.get(&event_url).send().await
            .map_err(|e| OpenCodeError::event_stream_error(format!("Failed to connect to SSE stream: {}", e)))?;
            
        if !response.status().is_success() {
            return Err(OpenCodeError::event_stream_error(format!("SSE endpoint returned status: {}", response.status())));
        }
        
        // Verify it's actually a SSE stream
        let content_type = response.headers().get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
            
        if !content_type.contains("text/event-stream") {
            return Err(OpenCodeError::event_stream_error(format!("Expected text/event-stream, got: {}", content_type)));
        }
        
        Ok(())
    }
    
    /// Process the SSE stream and parse events
    async fn process_sse_stream(config: &Configuration, sender: &broadcast::Sender<Event>) -> Result<()> {
        let event_url = format!("{}/event", config.base_path);
        let client = &config.client;
        
        let mut response = client.get(&event_url).send().await
            .map_err(|e| OpenCodeError::event_stream_error(format!("Failed to get SSE stream: {}", e)))?;
            
        // Process the streaming response
        while let Some(chunk) = response.chunk().await
            .map_err(|e| OpenCodeError::event_stream_error(format!("Failed to read SSE chunk: {}", e)))? {
            
            // Parse SSE format: "data: {JSON}\n"
            let chunk_str = std::str::from_utf8(&chunk)
                .map_err(|e| OpenCodeError::event_stream_error(format!("Invalid UTF-8 in SSE stream: {}", e)))?;
                
            for line in chunk_str.lines() {
                if let Some(event) = Self::parse_sse_line(line)? {
                    tracing::debug!("Parsed SSE event: {:?}", event);
                    
                    // Send event to all subscribers
                    if sender.send(event).is_err() {
                        tracing::debug!("No more receivers, stopping SSE stream");
                        return Ok(());
                    }
                }
            }
        }
        
        tracing::debug!("SSE stream ended");
        Ok(())
    }
    
    /// Parse a single SSE line and extract JSON event if present
    fn parse_sse_line(line: &str) -> Result<Option<Event>> {
        let trimmed = line.trim();
        
        // SSE format: "data: {JSON}"
        if let Some(data) = trimmed.strip_prefix("data: ") {
            if !data.trim().is_empty() {
                let event: Event = serde_json::from_str(data)
                    .map_err(|e| OpenCodeError::event_stream_error(format!("Failed to parse SSE JSON: {}", e)))?;
                return Ok(Some(event));
            }
        }
        
        // Ignore other SSE lines (comments, event types, etc.)
        Ok(None)
    }
}

/// Handle for receiving events from an event stream
#[derive(Debug)]
pub struct EventStreamHandle {
    receiver: broadcast::Receiver<Event>,
}

impl PartialEq for EventStreamHandle {
    fn eq(&self, _other: &Self) -> bool {
        // Since broadcast::Receiver doesn't implement PartialEq,
        // we'll consider all EventStreamHandles equal for now
        // In a real implementation, you might want to compare some unique identifier
        true
    }
}

impl EventStreamHandle {
    /// Receive the next event (blocking)
    pub async fn next_event(&mut self) -> Option<Event> {
        loop {
            match self.receiver.recv().await {
                Ok(event) => return Some(event),
                Err(broadcast::error::RecvError::Closed) => return None,
                Err(broadcast::error::RecvError::Lagged(_)) => {
                    // We lagged behind, continue to try to get the next event
                    continue;
                }
            }
        }
    }

    /// Try to receive an event without blocking
    pub fn try_next_event(&mut self) -> Option<Event> {
        loop {
            match self.receiver.try_recv() {
                Ok(event) => return Some(event),
                Err(broadcast::error::TryRecvError::Empty) => return None,
                Err(broadcast::error::TryRecvError::Closed) => return None,
                Err(broadcast::error::TryRecvError::Lagged(_)) => {
                    // We lagged behind, try again
                    continue;
                }
            }
        }
    }

    /// Check if the event stream is still active
    pub fn is_active(&self) -> bool {
        !self.receiver.is_closed()
    }
}

impl Clone for EventStreamHandle {
    fn clone(&self) -> Self {
        Self {
            receiver: self.receiver.resubscribe(),
        }
    }
}

