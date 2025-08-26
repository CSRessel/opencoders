//! Custom assertion helpers for smoke tests

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

// Tests temporarily removed due to SDK generation issues

