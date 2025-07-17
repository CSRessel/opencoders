//! Custom assertion helpers for smoke tests

use opencoders::sdk::{OpenCodeError, Result};

/// Assert that an API call succeeds, providing detailed error information on failure
#[macro_export]
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
#[macro_export]
macro_rules! assert_api_error {
    ($result:expr, $expected_error:pat, $context:expr) => {
        match $result {
            Ok(value) => {
                panic!("Expected error in {} but got success: {:?}", $context, value);
            }
            Err(e) => {
                if !matches!(e, $expected_error) {
                    panic!("Expected error pattern {} in {} but got: {:?}", 
                           stringify!($expected_error), $context, e);
                }
            }
        }
    };
}

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
pub fn validate_basic_response_structure<T>(result: &Result<T>, operation: &str) -> bool {
    match result {
        Ok(_) => true,
        Err(OpenCodeError::Http(_)) => {
            eprintln!("HTTP error in {}: network or connection issue", operation);
            false
        }
        Err(OpenCodeError::Serialization(_)) => {
            eprintln!("Serialization error in {}: API response format issue", operation);
            false
        }
        Err(OpenCodeError::Api { status, message }) => {
            eprintln!("API error in {}: {} - {}", operation, status, message);
            false
        }
        Err(e) => {
            eprintln!("Other error in {}: {}", operation, e);
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use opencoders::sdk::OpenCodeError;
    
    #[test]
    fn test_assert_api_success_macro() {
        let success_result: Result<String> = Ok("test".to_string());
        let value = assert_api_success!(success_result, "test operation");
        assert_eq!(value, "test");
    }
    
    #[test]
    #[should_panic(expected = "Test failed: test operation")]
    fn test_assert_api_success_macro_failure() {
        let error_result: Result<String> = Err(OpenCodeError::invalid_request("test error"));
        assert_api_success!(error_result, "test operation");
    }
    
    #[test]
    fn test_assert_api_error_macro() {
        let error_result: Result<String> = Err(OpenCodeError::invalid_request("test error"));
        assert_api_error!(error_result, OpenCodeError::InvalidRequest(_), "test operation");
    }
}