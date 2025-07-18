//! Basic connectivity smoke tests for the OpenCode SDK
//!
//! These tests verify that the generated SDK can successfully communicate
//! with a real opencode server instance for basic operations.

mod common;

use common::{assert_api_success, TestServer};
use opencoders::sdk::OpenCodeClient;

#[tokio::test]
async fn smoke_test_app_info() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");

    let client = OpenCodeClient::new(server.base_url());

    let app_info = client.get_app_info().await;
    let app = assert_api_success!(app_info, "get_app_info");

    // Verify basic app info structure
    common::assert_string_not_empty(&app.hostname, "app hostname");
    println!("✓ App info retrieved successfully: hostname {}", app.hostname);

    server.shutdown().await.expect("Failed to shutdown server");
}

#[tokio::test]
async fn smoke_test_config_endpoints() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");

    let client = OpenCodeClient::new(server.base_url());

    // Test config retrieval
    let config_result = client.get_config().await;
    let _config = assert_api_success!(config_result, "get_config");
    println!("✓ Config retrieved successfully");

    // Test providers list
    let providers_result = client.get_providers().await;
    let _providers = assert_api_success!(providers_result, "get_providers");
    println!("✓ Providers list retrieved successfully");

    // Test modes list
    let modes_result = client.get_modes().await;
    let modes = assert_api_success!(modes_result, "get_modes");
    common::assert_not_empty(&modes[..], "modes list");
    println!(
        "✓ Modes list retrieved successfully ({} modes)",
        modes.len()
    );

    server.shutdown().await.expect("Failed to shutdown server");
}

#[tokio::test]
async fn smoke_test_basic_connectivity_health() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");

    let client = OpenCodeClient::new(server.base_url());

    // Test multiple endpoints to ensure general connectivity
    
    // Test app info endpoint
    let app_result = client.get_app_info().await;
    assert!(
        common::validate_basic_response_structure(&app_result, "get_app_info"),
        "Endpoint get_app_info failed basic validation"
    );
    println!("✓ Endpoint get_app_info passed connectivity test");
    
    // Test config endpoint
    let config_result = client.get_config().await;
    assert!(
        common::validate_basic_response_structure(&config_result, "get_config"),
        "Endpoint get_config failed basic validation"
    );
    println!("✓ Endpoint get_config passed connectivity test");
    
    // Test modes endpoint
    let modes_result = client.get_modes().await;
    assert!(
        common::validate_basic_response_structure(&modes_result, "get_modes"),
        "Endpoint get_modes failed basic validation"
    );
    println!("✓ Endpoint get_modes passed connectivity test");

    server.shutdown().await.expect("Failed to shutdown server");
}

#[tokio::test]
async fn smoke_test_error_handling() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");

    // Test with invalid base URL to ensure error handling works
    let invalid_client = OpenCodeClient::new("http://localhost:99999");

    let result = invalid_client.get_app_info().await;
    assert!(result.is_err(), "Should fail with invalid server URL");

    let error = result.unwrap_err();
    assert!(
        error.is_retryable(),
        "Connection errors should be retryable"
    );
    println!("✓ Error handling works correctly for connection failures");

    server.shutdown().await.expect("Failed to shutdown server");
}

#[tokio::test]
async fn smoke_test_concurrent_requests() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");

    let _client = OpenCodeClient::new(server.base_url());

    // Test concurrent requests to ensure thread safety
    let task1 = tokio::spawn({
        let client = OpenCodeClient::new(server.base_url());
        async move { client.get_app_info().await }
    });
    let task2 = tokio::spawn({
        let client = OpenCodeClient::new(server.base_url());
        async move { client.get_config().await }
    });
    let task3 = tokio::spawn({
        let client = OpenCodeClient::new(server.base_url());
        async move { client.get_modes().await }
    });

    // Wait for all tasks to complete
    let result1 = task1.await.expect("Task should complete");
    assert!(result1.is_ok(), "Concurrent request 1 should succeed");
    println!("✓ Concurrent request 1 completed successfully");
    
    let result2 = task2.await.expect("Task should complete");
    assert!(result2.is_ok(), "Concurrent request 2 should succeed");
    println!("✓ Concurrent request 2 completed successfully");
    
    let result3 = task3.await.expect("Task should complete");
    assert!(result3.is_ok(), "Concurrent request 3 should succeed");
    println!("✓ Concurrent request 3 completed successfully");

    server.shutdown().await.expect("Failed to shutdown server");
}

