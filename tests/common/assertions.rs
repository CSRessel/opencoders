//! Custom assertion helpers for smoke tests

#![allow(dead_code)]

/// Assert that an API call succeeds, providing detailed error information on failure
#[macro_export()]
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
#[macro_export()]
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

/// Assert that a string is not empty
pub fn assert_string_not_empty(s: &str, context: &str) {
    assert!(!s.is_empty(), "{} should not be empty", context);
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
