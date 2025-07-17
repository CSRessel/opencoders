//! Basic connectivity smoke tests for the OpenCode SDK
//! 
//! These tests verify that the generated SDK can successfully communicate
//! with a real opencode server instance for basic operations.

mod common;

use common::TestServer;
use opencoders::sdk::OpenCodeClient;

#[macro_use]
extern crate opencoders;

#[tokio::test]
async fn smoke_test_app_info() {
    let server = TestServer::start().await
        .expect("Failed to start test server");
    
    let client = OpenCodeClient::new(server.base_url());
    
    let app_info = client.get_app_info().await;
    let app = assert_api_success!(app_info, "get_app_info");
    
    // Verify basic app info structure
    common::assert_string_not_empty(&app.version, "app version");
    println!("✓ App info retrieved successfully: version {}", app.version);
    
    server.shutdown().await.expect("Failed to shutdown server");
}

#[tokio::test]
async fn smoke_test_config_endpoints() {
    let server = TestServer::start().await
        .expect("Failed to start test server");
    
    let client = OpenCodeClient::new(server.base_url());
    
    // Test config retrieval
    let config_result = client.get_config().await;
    let config = assert_api_success!(config_result, "get_config");
    println!("✓ Config retrieved successfully");
    
    // Test providers list
    let providers_result = client.get_providers().await;
    let providers = assert_api_success!(providers_result, "get_providers");
    println!("✓ Providers list retrieved successfully");
    
    // Test modes list
    let modes_result = client.get_modes().await;
    let modes = assert_api_success!(modes_result, "get_modes");
    common::assert_not_empty(&modes, "modes list");
    println!("✓ Modes list retrieved successfully ({} modes)", modes.len());
    
    server.shutdown().await.expect("Failed to shutdown server");
}

#[tokio::test]
async fn smoke_test_basic_connectivity_health() {
    let server = TestServer::start().await
        .expect("Failed to start test server");
    
    let client = OpenCodeClient::new(server.base_url());
    
    // Test multiple endpoints to ensure general connectivity
    let endpoints = vec![
        ("get_app_info", || Box::pin(client.get_app_info())),
        ("get_config", || Box::pin(client.get_config())),
        ("get_modes", || Box::pin(client.get_modes())),
    ];
    
    for (name, endpoint_fn) in endpoints {
        let result = endpoint_fn().await;
        assert!(
            common::validate_basic_response_structure(&result, name),
            "Endpoint {} failed basic validation", name
        );
        println!("✓ Endpoint {} passed connectivity test", name);
    }
    
    server.shutdown().await.expect("Failed to shutdown server");
}

#[tokio::test]
async fn smoke_test_error_handling() {
    let server = TestServer::start().await
        .expect("Failed to start test server");
    
    // Test with invalid base URL to ensure error handling works
    let invalid_client = OpenCodeClient::new("http://localhost:99999");
    
    let result = invalid_client.get_app_info().await;
    assert!(result.is_err(), "Should fail with invalid server URL");
    
    let error = result.unwrap_err();
    assert!(error.is_retryable(), "Connection errors should be retryable");
    println!("✓ Error handling works correctly for connection failures");
    
    server.shutdown().await.expect("Failed to shutdown server");
}

#[tokio::test]
async fn smoke_test_concurrent_requests() {
    let server = TestServer::start().await
        .expect("Failed to start test server");
    
    let client = OpenCodeClient::new(server.base_url());
    
    // Test concurrent requests to ensure thread safety
    let tasks = vec![
        tokio::spawn({
            let client = OpenCodeClient::new(server.base_url());
            async move { client.get_app_info().await }
        }),
        tokio::spawn({
            let client = OpenCodeClient::new(server.base_url());
            async move { client.get_config().await }
        }),
        tokio::spawn({
            let client = OpenCodeClient::new(server.base_url());
            async move { client.get_modes().await }
        }),
    ];
    
    // Wait for all tasks to complete
    for (i, task) in tasks.into_iter().enumerate() {
        let result = task.await.expect("Task should complete");
        assert!(result.is_ok(), "Concurrent request {} should succeed", i);
        println!("✓ Concurrent request {} completed successfully", i);
    }
    
    server.shutdown().await.expect("Failed to shutdown server");
}