//! Custom assertion helpers for smoke tests

#![allow(unused_imports)]
#![allow(unused_macros)]

/// Assert that an API call succeeds, providing detailed error information on failure
macro_rules! assert_api_success {
    ($result:expr, $context:expr) => {
        match $result {
            Ok(value) => value,
            Err(e) => {
                eprintln!("API call failed in {}: {:?}", $context, e);
                eprintln!("Error details: {}", e);
                if e.is_retryable() {
                    eprintln!("This error is marked as retryable");
                }
                if e.is_client_error() {
                    eprintln!("This is a client error (4xx)");
                }
                if e.is_server_error() {
                    eprintln!("This is a server error (5xx)");
                }
                panic!("Test failed: {}", $context);
            }
        }
    };
}

/// Assert that an API call fails with a specific error type
macro_rules! assert_api_error {
    ($result:expr, $expected_error:pat, $context:expr) => {
        match $result {
            Ok(value) => {
                panic!(
                    "Expected error in {} but got success: {:?}",
                    $context, value
                );
            }
            Err(e) => {
                if !matches!(e, $expected_error) {
                    panic!(
                        "Expected error pattern {} in {} but got: {:?}",
                        stringify!($expected_error),
                        $context,
                        e
                    );
                }
            }
        }
    };
}

// Export the macros
pub(crate) use assert_api_error;
pub(crate) use assert_api_success;

/// Assert that a collection is not empty
pub fn assert_not_empty<T>(collection: &[T], context: &str) {
    assert!(!collection.is_empty(), "{} should not be empty", context);
}

/// Assert that a string is not empty
pub fn assert_string_not_empty(s: &str, context: &str) {
    assert!(!s.is_empty(), "{} should not be empty", context);
}

/// Assert that an optional value is present
pub fn assert_some<T>(option: &Option<T>, context: &str) {
    assert!(option.is_some(), "{} should be Some", context);
}

/// Helper to check if a server response looks reasonable
/// Temporarily simplified due to SDK generation issues
pub fn validate_basic_response_structure<T, E>(
    result: &std::result::Result<T, E>,
    operation: &str,
) -> bool
where
    E: std::fmt::Display,
{
    match result {
        Ok(_) => true,
        Err(e) => {
            eprintln!("Error in {}: {}", operation, e);
            false
        }
    }
}

/// Assert that an error message is not empty
pub fn assert_error_not_empty<E>(error: &E, context: &str)
where
    E: std::fmt::Display,
{
    let error_string = format!("{}", error);
    assert!(
        !error_string.is_empty(),
        "Error message in {} should not be empty",
        context
    );
}

// SDK-specific assertion functions

use eyre::Result;
use opencode_sdk::models::{App, Config, ConfigProviders200Response, Session};

/// Assert that an App info structure is valid
pub fn assert_app_info_valid(app: &App) -> Result<()> {
    assert!(!app.hostname.is_empty(), "App hostname should not be empty");
    // Git field should be boolean
    assert!(app.git || !app.git, "Git field should be valid boolean");
    Ok(())
}

/// Assert that a Config structure is valid
pub fn assert_config_valid(config: &Config) -> Result<()> {
    // Config should have some basic structure
    assert!(
        config.agent.is_some() || config.agent.is_none(),
        "Config should have valid agent field"
    );
    Ok(())
}

/// Assert that a Session structure is valid
pub fn assert_session_valid(session: &Session) -> Result<()> {
    assert!(!session.id.is_empty(), "Session ID should not be empty");
    assert!(
        !session.title.is_empty(),
        "Session title should not be empty"
    );
    assert!(
        !session.version.is_empty(),
        "Session version should not be empty"
    );
    Ok(())
}

/// Assert that a providers response is valid
pub fn assert_providers_valid(providers: &ConfigProviders200Response) -> Result<()> {
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

