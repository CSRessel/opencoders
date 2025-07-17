//! Session lifecycle smoke tests for the OpenCode SDK
//! 
//! These tests verify that session management operations work correctly
//! with a real opencode server instance.

mod common;

use common::TestServer;
use opencoders::sdk::OpenCodeClient;

#[macro_use]
extern crate opencoders;

#[tokio::test]
async fn smoke_test_session_list_empty() {
    let server = TestServer::start().await
        .expect("Failed to start test server");
    
    let client = OpenCodeClient::new(server.base_url());
    
    // List sessions (should work even if empty)
    let sessions_result = client.list_sessions().await;
    let sessions = assert_api_success!(sessions_result, "list_sessions");
    
    // Should return an empty list initially
    println!("✓ Session list retrieved successfully ({} sessions)", sessions.len());
    
    server.shutdown().await.expect("Failed to shutdown server");
}

#[tokio::test]
async fn smoke_test_session_create_and_delete() {
    let server = TestServer::start().await
        .expect("Failed to start test server");
    
    let client = OpenCodeClient::new(server.base_url());
    
    // Create new session
    let session_result = client.create_session().await;
    let session = assert_api_success!(session_result, "create_session");
    
    common::assert_string_not_empty(&session.id, "session ID");
    println!("✓ Session created successfully with ID: {}", session.id);
    
    // Verify session exists in list
    let sessions_result = client.list_sessions().await;
    let sessions = assert_api_success!(sessions_result, "list_sessions after create");
    
    let found_session = sessions.iter().find(|s| s.id == session.id);
    assert!(found_session.is_some(), "Created session should appear in session list");
    println!("✓ Created session found in session list");
    
    // Delete the session
    let delete_result = client.delete_session(&session.id).await;
    let deleted = assert_api_success!(delete_result, "delete_session");
    assert!(deleted, "Delete operation should return true");
    println!("✓ Session deleted successfully");
    
    // Verify session no longer exists in list
    let sessions_after_delete = client.list_sessions().await;
    let sessions_after = assert_api_success!(sessions_after_delete, "list_sessions after delete");
    
    let found_after_delete = sessions_after.iter().find(|s| s.id == session.id);
    assert!(found_after_delete.is_none(), "Deleted session should not appear in session list");
    println!("✓ Deleted session no longer appears in session list");
    
    server.shutdown().await.expect("Failed to shutdown server");
}

#[tokio::test]
async fn smoke_test_session_operations() {
    let server = TestServer::start().await
        .expect("Failed to start test server");
    
    let client = OpenCodeClient::new(server.base_url());
    
    // Create a session for testing operations
    let session_result = client.create_session().await;
    let session = assert_api_success!(session_result, "create_session");
    
    // Test abort session (should work even if session isn't running)
    let abort_result = client.abort_session(&session.id).await;
    let aborted = assert_api_success!(abort_result, "abort_session");
    println!("✓ Session abort operation completed: {}", aborted);
    
    // Test getting messages for the session (should be empty initially)
    let messages_result = client.get_messages(&session.id).await;
    let messages = assert_api_success!(messages_result, "get_messages");
    println!("✓ Messages retrieved for session ({} messages)", messages.len());
    
    // Clean up
    let delete_result = client.delete_session(&session.id).await;
    assert_api_success!(delete_result, "cleanup delete_session");
    
    server.shutdown().await.expect("Failed to shutdown server");
}

#[tokio::test]
async fn smoke_test_multiple_sessions() {
    let server = TestServer::start().await
        .expect("Failed to start test server");
    
    let client = OpenCodeClient::new(server.base_url());
    
    // Create multiple sessions
    let mut session_ids = Vec::new();
    for i in 0..3 {
        let session_result = client.create_session().await;
        let session = assert_api_success!(session_result, &format!("create_session_{}", i));
        session_ids.push(session.id);
        println!("✓ Created session {}: {}", i + 1, session_ids[i]);
    }
    
    // Verify all sessions exist
    let sessions_result = client.list_sessions().await;
    let sessions = assert_api_success!(sessions_result, "list_sessions");
    
    for session_id in &session_ids {
        let found = sessions.iter().any(|s| s.id == *session_id);
        assert!(found, "Session {} should exist in list", session_id);
    }
    println!("✓ All {} sessions found in session list", session_ids.len());
    
    // Clean up all sessions
    for (i, session_id) in session_ids.iter().enumerate() {
        let delete_result = client.delete_session(session_id).await;
        assert_api_success!(delete_result, &format!("delete_session_{}", i));
        println!("✓ Deleted session {}: {}", i + 1, session_id);
    }
    
    // Verify all sessions are gone
    let final_sessions_result = client.list_sessions().await;
    let final_sessions = assert_api_success!(final_sessions_result, "final list_sessions");
    
    for session_id in &session_ids {
        let found = final_sessions.iter().any(|s| s.id == *session_id);
        assert!(!found, "Session {} should not exist after deletion", session_id);
    }
    println!("✓ All sessions successfully deleted");
    
    server.shutdown().await.expect("Failed to shutdown server");
}

#[tokio::test]
async fn smoke_test_session_error_handling() {
    let server = TestServer::start().await
        .expect("Failed to start test server");
    
    let client = OpenCodeClient::new(server.base_url());
    
    // Test operations on non-existent session
    let fake_session_id = "non-existent-session-id";
    
    // These operations should fail gracefully
    let delete_result = client.delete_session(fake_session_id).await;
    assert!(delete_result.is_err(), "Deleting non-existent session should fail");
    
    let messages_result = client.get_messages(fake_session_id).await;
    assert!(messages_result.is_err(), "Getting messages for non-existent session should fail");
    
    let abort_result = client.abort_session(fake_session_id).await;
    assert!(abort_result.is_err(), "Aborting non-existent session should fail");
    
    println!("✓ Error handling works correctly for non-existent sessions");
    
    server.shutdown().await.expect("Failed to shutdown server");
}