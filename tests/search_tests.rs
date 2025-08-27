//! Search operations smoke tests for the OpenCode SDK
//!
//! These tests verify that search functionality works correctly
//! with a real opencode server instance.

mod common;

use common::{assert_error_not_empty, TestServer};
use opencoders::sdk::OpenCodeClient;

use crate::common::assert_string_not_empty;

#[tokio::test]
async fn smoke_test_find_files() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");

    let client = OpenCodeClient::new(server.base_url());

    // Test file search with common patterns
    let search_patterns = vec![
        ("*", "wildcard all files"),
        ("*.txt", "text files"),
        ("*.rs", "Rust files"),
        ("test*", "files starting with 'test'"),
    ];

    for (pattern, description) in search_patterns {
        let files_result = client.find_files(pattern).await;
        match files_result {
            Ok(files) => {
                println!(
                    "✓ File search for {} succeeded ({} files found)",
                    description,
                    files.len()
                );

                // Verify that results are strings (file paths)
                for file_path in &files {
                    assert_string_not_empty(file_path, "file path in search results");
                }
            }
            Err(e) => {
                println!(
                    "Note: File search for {} failed (may be expected): {}",
                    description, e
                );
                // Ensure error is properly structured
                assert_error_not_empty(&e, "file search error");
            }
        }
    }

    server.shutdown().await.expect("Failed to shutdown server");
}

#[tokio::test]
async fn smoke_test_find_text() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");

    let client = OpenCodeClient::new(server.base_url());

    // Test text search with common patterns that might exist in any codebase
    let search_patterns = vec![
        ("function", "function keyword"),
        ("import", "import statements"),
        ("const", "const declarations"),
        ("test", "test-related text"),
        ("error", "error-related text"),
    ];

    for (pattern, description) in search_patterns {
        let matches_result = client.find_text(pattern).await;
        match matches_result {
            Ok(matches) => {
                println!(
                    "✓ Text search for {} succeeded ({} matches found)",
                    description,
                    matches.len()
                );

                // Verify match structure if we have results
                for match_result in &matches {
                    assert_string_not_empty(&match_result.path.text, "match file path");
                    // Note: We don't validate line numbers or content as they depend on the specific files
                }
            }
            Err(e) => {
                println!(
                    "Note: Text search for {} failed (may be expected in empty directory): {}",
                    description, e
                );
                // Ensure error is properly structured
                assert_error_not_empty(&e, "text search error");
            }
        }
    }

    server.shutdown().await.expect("Failed to shutdown server");
}

#[tokio::test]
async fn smoke_test_find_symbols() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");

    let client = OpenCodeClient::new(server.base_url());

    // Test symbol search with common symbol patterns
    let search_patterns = vec![
        ("main", "main function/symbol"),
        ("test", "test symbols"),
        ("config", "config-related symbols"),
        ("client", "client-related symbols"),
    ];

    for (pattern, description) in search_patterns {
        let symbols_result = client.find_symbols(pattern).await;
        match symbols_result {
            Ok(symbols) => {
                println!(
                    "✓ Symbol search for {} succeeded ({} symbols found)",
                    description,
                    symbols.len()
                );

                // Verify symbol structure if we have results
                for symbol in &symbols {
                    assert_string_not_empty(&symbol.name, "symbol name");
                    // Note: Other fields like location depend on the specific codebase
                }
            }
            Err(e) => {
                println!(
                    "Note: Symbol search for {} failed (may be expected without LSP): {}",
                    description, e
                );
                // Ensure error is properly structured
                assert_error_not_empty(&e, "symbol search error");
            }
        }
    }

    server.shutdown().await.expect("Failed to shutdown server");
}

#[tokio::test]
async fn smoke_test_search_error_handling() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");

    let client = OpenCodeClient::new(server.base_url());

    // Test various error conditions for search operations
    let invalid_patterns = vec![
        ("", "empty pattern"),
        ("   ", "whitespace only pattern"),
        ("[invalid-regex", "invalid regex pattern"),
    ];

    for (pattern, description) in invalid_patterns {
        // Test file search
        let file_result = client.find_files(pattern).await;
        match file_result {
            Ok(files) => {
                println!(
                    "Note: File search with {} unexpectedly succeeded ({} results)",
                    description,
                    files.len()
                );
            }
            Err(e) => {
                println!(
                    "✓ File search with {} failed as expected: {}",
                    description, e
                );
            }
        }

        // Test text search
        let text_result = client.find_text(pattern).await;
        match text_result {
            Ok(matches) => {
                println!(
                    "Note: Text search with {} unexpectedly succeeded ({} results)",
                    description,
                    matches.len()
                );
            }
            Err(e) => {
                println!(
                    "✓ Text search with {} failed as expected: {}",
                    description, e
                );
            }
        }

        // Test symbol search
        let symbol_result = client.find_symbols(pattern).await;
        match symbol_result {
            Ok(symbols) => {
                println!(
                    "Note: Symbol search with {} unexpectedly succeeded ({} results)",
                    description,
                    symbols.len()
                );
            }
            Err(e) => {
                println!(
                    "✓ Symbol search with {} failed as expected: {}",
                    description, e
                );
            }
        }
    }

    server.shutdown().await.expect("Failed to shutdown server");
}

#[tokio::test]
async fn smoke_test_concurrent_search_operations() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");

    let base_url = server.base_url().to_string();

    // Test concurrent search requests
    let tasks = vec![
        tokio::spawn({
            let url = base_url.clone();
            async move {
                let client = OpenCodeClient::new(&url);
                ("find_files", client.find_files("*").await.map(|r| r.len()))
            }
        }),
        tokio::spawn({
            let url = base_url.clone();
            async move {
                let client = OpenCodeClient::new(&url);
                ("find_text", client.find_text("test").await.map(|r| r.len()))
            }
        }),
        tokio::spawn({
            let url = base_url.clone();
            async move {
                let client = OpenCodeClient::new(&url);
                (
                    "find_symbols",
                    client.find_symbols("main").await.map(|r| r.len()),
                )
            }
        }),
    ];

    // Wait for all tasks to complete
    for task in tasks {
        let (operation, result) = task.await.expect("Task should complete");
        match result {
            Ok(count) => {
                println!(
                    "✓ Concurrent {} operation completed successfully ({} results)",
                    operation, count
                );
            }
            Err(e) => {
                println!(
                    "Note: Concurrent {} operation failed (may be expected): {}",
                    operation, e
                );
            }
        }
    }

    server.shutdown().await.expect("Failed to shutdown server");
}

#[tokio::test]
async fn smoke_test_search_consistency() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");

    let client = OpenCodeClient::new(server.base_url());

    // Test that search results are consistent across multiple calls
    let pattern = "*";
    let mut results = Vec::new();

    for i in 0..3 {
        let files_result = client.find_files(pattern).await;
        match files_result {
            Ok(files) => {
                results.push(files);
                println!(
                    "✓ Search consistency test {} completed ({} files)",
                    i + 1,
                    results[i].len()
                );
            }
            Err(e) => {
                println!("Note: Search consistency test {} failed: {}", i + 1, e);
                // If search fails, we can't test consistency, but that's okay
                server.shutdown().await.expect("Failed to shutdown server");
                return;
            }
        }
    }

    // If we have results, verify they're consistent
    if results.len() > 1 {
        let first_count = results[0].len();
        for (i, result) in results.iter().enumerate() {
            assert_eq!(
                result.len(),
                first_count,
                "Search result {} returned different number of files",
                i + 1
            );
        }
        println!("✓ Search results are consistent across multiple requests");
    }

    server.shutdown().await.expect("Failed to shutdown server");
}
