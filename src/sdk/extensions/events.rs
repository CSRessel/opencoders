//! Event stream handling for real-time updates

use crate::sdk::error::Result;
use opencode_sdk::{apis::{configuration::Configuration, default_api}, models::Event};
use std::time::Duration;
use tokio::sync::broadcast;
use tokio::time::interval;

/// Event stream for receiving real-time updates from the OpenCode server
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

    /// Internal polling loop for events
    async fn poll_events(config: Configuration, sender: broadcast::Sender<Event>) {
        let mut interval = interval(Duration::from_millis(100)); // Poll every 100ms
        let mut consecutive_errors = 0;
        const MAX_CONSECUTIVE_ERRORS: u32 = 10;

        loop {
            interval.tick().await;

            match default_api::get_event(&config).await {
                Ok(event) => {
                    consecutive_errors = 0;

                    // Send event to all subscribers
                    if sender.send(event).is_err() {
                        // No more receivers, exit the loop
                        break;
                    }
                }
                Err(e) => {
                    consecutive_errors += 1;
                    eprintln!("Event polling error ({}): {}", consecutive_errors, e);

                    if consecutive_errors >= MAX_CONSECUTIVE_ERRORS {
                        eprintln!("Too many consecutive errors, stopping event stream");
                        break;
                    }

                    // Exponential backoff on errors
                    let backoff_duration =
                        Duration::from_millis(100 * (2_u64.pow(consecutive_errors.min(6))));
                    tokio::time::sleep(backoff_duration).await;
                }
            }
        }
    }
}

/// Handle for receiving events from an event stream
pub struct EventStreamHandle {
    receiver: broadcast::Receiver<Event>,
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

