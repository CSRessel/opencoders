//! Simple smoke test for OpenCode server connectivity
//! 
//! This test verifies basic server connectivity without relying on the generated SDK

mod common;

use common::{TestServer, assert_string_not_empty};
use reqwest::Client;
use serde_json::Value;

#[tokio::test]
async fn test_server_starts_and_responds() {
    let server = TestServer::start().await
        .expect("Failed to start test server");
    
    let client = Client::new();
    
    // Test basic connectivity to /app endpoint
    let response = client.get(&format!("{}/app", server.base_url())).send().await;
    assert!(response.is_ok(), "Should be able to connect to /app endpoint");
    
    let response = response.unwrap();
    assert!(response.status().is_success(), "App endpoint should return success status");
    
    println!("✓ Server connectivity test passed");
    
    server.shutdown().await.expect("Failed to shutdown server");
}

#[tokio::test]
async fn test_basic_endpoints_respond() {
    let server = TestServer::start().await
        .expect("Failed to start test server");
    
    let client = Client::new();
    let base_url = server.base_url();
    
    // Test multiple endpoints
    let endpoints = vec![
        "/app",
        "/config", 
        "/mode",
        "/session",
    ];
    
    for endpoint in endpoints {
        let url = format!("{}{}", base_url, endpoint);
        println!("Testing endpoint: {}", url);
        
        let response = client.get(&url).send().await;
        assert!(response.is_ok(), "Should be able to connect to {}", endpoint);
        
        let response = response.unwrap();
        println!("  Status: {}", response.status());
        
        // We expect either success or a well-formed error response
        assert!(
            response.status().is_success() || response.status().is_client_error(),
            "Endpoint {} should return success or client error, got: {}", 
            endpoint, 
            response.status()
        );
    }
    
    println!("✓ Basic endpoints test passed");
    
    server.shutdown().await.expect("Failed to shutdown server");
}

#[tokio::test]
async fn test_json_response_format() {
    let server = TestServer::start().await
        .expect("Failed to start test server");
    
    let client = Client::new();
    
    // Test that /app returns valid JSON
    let response = client.get(&format!("{}/app", server.base_url())).send().await
        .expect("Should be able to connect to /app");
    
    if response.status().is_success() {
        let text = response.text().await.expect("Should be able to read response text");
        let json: Result<Value, _> = serde_json::from_str(&text);
        assert!(json.is_ok(), "App endpoint should return valid JSON: {}", text);
        
        let json = json.unwrap();
        println!("App response: {}", serde_json::to_string_pretty(&json).unwrap());
        
        // Basic validation - should have version field
        if let Some(version) = json.get("version") {
            assert!(version.is_string(), "Version should be a string");
            if let Some(version_str) = version.as_str() {
                assert_string_not_empty(version_str, "version field");
            }
            println!("✓ Found version: {}", version);
        }
    } else {
        println!("App endpoint returned non-success status: {}", response.status());
    }
    
    println!("✓ JSON response format test passed");
    
    server.shutdown().await.expect("Failed to shutdown server");
}

#[tokio::test]
async fn test_concurrent_requests() {
    let server = TestServer::start().await
        .expect("Failed to start test server");
    
    let base_url = server.base_url().to_string();
    
    // Test concurrent requests
    let tasks = (0..5).map(|i| {
        let url = base_url.clone();
        tokio::spawn(async move {
            let client = Client::new();
            let response = client.get(&format!("{}/app", url)).send().await;
            (i, response)
        })
    }).collect::<Vec<_>>();
    
    // Wait for all tasks to complete
    for task in tasks {
        let (task_id, result) = task.await.expect("Task should complete");
        assert!(result.is_ok(), "Concurrent request {} should succeed", task_id);
        println!("✓ Concurrent request {} completed successfully", task_id);
    }
    
    println!("✓ Concurrent requests test passed");
    
    server.shutdown().await.expect("Failed to shutdown server");
}

#[tokio::test]
async fn test_server_lifecycle() {
    // Test that we can start and stop multiple servers
    for i in 0..3 {
        println!("Starting server instance {}", i + 1);
        
        let server = TestServer::start().await
            .expect(&format!("Failed to start test server {}", i + 1));
        
        let client = Client::new();
        let response = client.get(&format!("{}/app", server.base_url())).send().await;
        assert!(response.is_ok(), "Server {} should respond", i + 1);
        
        server.shutdown().await
            .expect(&format!("Failed to shutdown server {}", i + 1));
        
        println!("✓ Server instance {} lifecycle completed", i + 1);
    }
    
    println!("✓ Server lifecycle test passed");
}