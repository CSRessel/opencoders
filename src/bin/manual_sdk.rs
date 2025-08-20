#!/usr/bin/env rust

//! Test binary to fetch messages for a specific session ID and analyze event synchronization
//!
//! Usage: cargo run --bin test_session_messages

use opencode_sdk::{
    apis::default_api,
    models::{Event, Part, ToolState},
};
use opencoders::sdk::{extensions::events::EventStream, OpenCodeClient};
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging (ignore errors for this test binary)
    let _ = opencoders::app::logger::init();

    println!("=== OpenCode Session Messages Test ===");
    println!();

    // Hardcoded session ID for testing

    // small debugging session:
    // let session_id = "ses_73c208b19ffeJuppPVZMiFVdBb";

    // multiple types of tool calls:
    let session_id = "ses_7372bba2cffejwScF7qXSbOaXc";

    println!("Testing session ID: {}", session_id);
    println!();

    // Initialize client the same way as app_program.rs (line 295)
    println!("🔍 Discovering OpenCode server...");
    let start_time = Instant::now();

    let client = match OpenCodeClient::discover().await {
        Ok(client) => {
            println!("✅ Connected to server at: {}", client.base_url());
            println!("⏱️  Connection time: {:?}", start_time.elapsed());
            client
        }
        Err(e) => {
            eprintln!("❌ Failed to connect to OpenCode server: {}", e);
            eprintln!("   Make sure the OpenCode server is running (usually on port 41100)");
            eprintln!("   You can also set OPENCODE_SERVER_URL environment variable");
            return Err(e.into());
        }
    };

    println!();

    // Test 1: Fetch messages for the session
    println!("📥 Fetching messages for session {}...", session_id);
    let fetch_start = Instant::now();

    match client.get_messages(session_id).await {
        Ok(messages) => {
            let fetch_time = fetch_start.elapsed();
            println!("✅ Successfully fetched {} messages", messages.len());
            println!("⏱️  Fetch time: {:?}", fetch_time);
            println!();

            if messages.is_empty() {
                println!("ℹ️  No messages found in this session");
                println!("   This could mean:");
                println!("   - Session doesn't exist");
                println!("   - Session has no messages yet");
                println!("   - Session ID is incorrect");
                return Ok(());
            }

            // Analyze messages for event sync issues
            analyze_messages_for_sync_issues(&messages);
        }
        Err(e) => {
            eprintln!("❌ Failed to fetch messages: {}", e);
            eprintln!("   Error details: {:?}", e);
            return Err(e.into());
        }
    }

    println!();

    // Test 2: Test the new SSE implementation
    test_new_sse_implementation(&client).await?;

    println!();
    println!("=== Test Complete ===");

    Ok(())
}

fn analyze_messages_for_sync_issues(
    messages: &[opencode_sdk::models::GetSessionByIdMessage200ResponseInner],
) {
    println!(
        "raw:\n{}\n\n",
        serde_json::to_string_pretty(messages).unwrap()
    );

    println!("🔍 Analyzing messages for synchronization issues...");
    println!();

    let mut total_parts = 0;
    let mut tool_parts = 0;
    let mut pending_tools = 0;
    let mut running_tools = 0;
    let mut completed_tools = 0;
    let mut error_tools = 0;

    for (msg_idx, message_container) in messages.iter().enumerate() {
        let _message_info = &message_container.info;
        let parts = &message_container.parts;

        println!("Message {}: {} parts", msg_idx + 1, parts.len());

        for (part_idx, part) in parts.iter().enumerate() {
            total_parts += 1;

            match part {
                Part::Tool(tool_part) => {
                    tool_parts += 1;
                    println!(
                        "  Part {}: Tool '{}' (ID: {})",
                        part_idx + 1,
                        tool_part.tool,
                        tool_part.id,
                    );

                    match tool_part.state.as_ref() {
                        ToolState::Pending(pending) => {
                            pending_tools += 1;
                            println!("    Status: PENDING ({})", pending.status);
                        }
                        ToolState::Running(running) => {
                            running_tools += 1;
                            println!("    Status: RUNNING ({})", running.status);
                        }
                        ToolState::Completed(completed) => {
                            completed_tools += 1;
                            println!("    Status: COMPLETED ({})", completed.status);
                            println!("    Output: {}", completed.output);
                        }
                        ToolState::Error(error) => {
                            error_tools += 1;
                            println!("    Status: ERROR ({})", error.status);
                        }
                    }
                }
                Part::Text(text_part) => {
                    println!(
                        "  Part {}: Text (chars {})",
                        part_idx + 1,
                        text_part.text.len()
                    );
                }
                Part::File(file_part) => {
                    println!(
                        "  Part {}: File (name {})",
                        part_idx + 1,
                        file_part.filename.clone().unwrap_or("-".to_string()),
                    );
                }
                // Part::StepStart(step_part) => {
                //     println!(
                //         "  Part {}: Step Start (preview {})",
                //         part_idx + 1,
                //         serde_json::to_string(step_part).unwrap(),
                //     );
                // }
                // Part::StepFinish(_step_part) => {
                //     println!("  Part {}: Step Finish", part_idx + 1);
                // }
                Part::Snapshot(snapshot_part) => {
                    println!(
                        "  Part {}: Snapshot (snap {})",
                        part_idx + 1,
                        snapshot_part.snapshot,
                    );
                }
                _ => (),
            }
        }
        println!();
    }

    // Summary statistics
    println!("📊 Message Analysis Summary:");
    println!("  Total parts: {}", total_parts);
    println!("  Tool parts: {}", tool_parts);

    if tool_parts > 0 {
        println!("  Tool states:");
        println!(
            "    Pending: {} ({:.1}%)",
            pending_tools,
            (pending_tools as f64 / tool_parts as f64) * 100.0
        );
        println!(
            "    Running: {} ({:.1}%)",
            running_tools,
            (running_tools as f64 / tool_parts as f64) * 100.0
        );
        println!(
            "    Completed: {} ({:.1}%)",
            completed_tools,
            (completed_tools as f64 / tool_parts as f64) * 100.0
        );
        println!(
            "    Error: {} ({:.1}%)",
            error_tools,
            (error_tools as f64 / tool_parts as f64) * 100.0
        );
        println!("   {} tools are in PENDING state", pending_tools);
    }
}

async fn test_new_sse_implementation(
    client: &OpenCodeClient,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing NEW SSE implementation from events.rs...");
    println!();

    // Create EventStream instance
    let event_stream = EventStream::new(client.configuration().clone()).await?;
    let mut event_handle = event_stream.handle();

    println!("📡 Starting SSE event stream...");
    let start_time = Instant::now();

    // Test the stream for a limited time
    let mut event_count = 0;
    let max_duration = std::time::Duration::from_secs(5);
    let max_events = 10;

    let result = tokio::time::timeout(max_duration, async {
        while let Some(event) = event_handle.next_event().await {
            event_count += 1;
            println!("✅ Received event #{}", event_count);

            // Pretty print the event with detailed formatting
            pretty_print_event(&event);
            println!();

            if event_count >= max_events {
                println!("🛑 Stopping after {} events (test limit)", max_events);
                break;
            }
        }

        Ok::<(), Box<dyn std::error::Error>>(())
    })
    .await;

    let elapsed = start_time.elapsed();

    match result {
        Ok(Ok(())) => {
            println!("✅ SSE stream test completed successfully!");
        }
        Ok(Err(e)) => {
            println!("❌ SSE stream encountered an error: {}", e);
        }
        Err(_) => {
            println!("⏰ SSE stream test timed out after {:?}", max_duration);
        }
    }

    println!("📊 SSE Stream Test Results:");
    println!("  Events received: {}", event_count);
    println!("  Total time: {:?}", elapsed);
    println!(
        "  Events per second: {:.2}",
        event_count as f64 / elapsed.as_secs_f64()
    );
    println!();

    if event_count > 0 {
        println!("🎉 SUCCESS: New SSE implementation is working!");
    } else {
        println!("⚠️  WARNING: No events received");
    }

    Ok(())
}

fn pretty_print_event(event: &Event) {
    println!("📋 Event Details:");

    // Print event type first
    match event {
        Event::MessagePeriodUpdated(msg_event) => {
            println!("  🔄 Event Type: MessageUpdated");
            println!("  📝 Type: {}", msg_event.r#type);

            // Pretty print the properties as JSON
            if let Ok(json_str) = serde_json::to_string_pretty(&msg_event.properties) {
                println!("  📄 Properties:");
                for line in json_str.lines() {
                    println!("    {}", line);
                }
            }
        }
        Event::MessagePeriodPartPeriodUpdated(part_event) => {
            println!("  🔄 Event Type: MessagePartUpdated");
            println!("  📝 Type: {}", part_event.r#type);

            // Pretty print the properties as JSON
            if let Ok(json_str) = serde_json::to_string_pretty(&part_event.properties) {
                println!("  📄 Properties:");
                for line in json_str.lines() {
                    println!("    {}", line);
                }
            }
        }
        Event::MessagePeriodRemoved(remove_event) => {
            println!("  🗑️  Event Type: MessageRemoved");
            println!("  📝 Type: {}", remove_event.r#type);

            // Pretty print the properties as JSON
            if let Ok(json_str) = serde_json::to_string_pretty(&remove_event.properties) {
                println!("  📄 Properties:");
                for line in json_str.lines() {
                    println!("    {}", line);
                }
            }
        }
        Event::SessionPeriodUpdated(session_event) => {
            println!("  🔄 Event Type: SessionUpdated");
            println!("  📝 Type: {}", session_event.r#type);

            // Pretty print the properties as JSON
            if let Ok(json_str) = serde_json::to_string_pretty(&session_event.properties) {
                println!("  📄 Properties:");
                for line in json_str.lines() {
                    println!("    {}", line);
                }
            }
        }
        Event::SessionPeriodDeleted(delete_event) => {
            println!("  🗑️  Event Type: SessionDeleted");
            println!("  📝 Type: {}", delete_event.r#type);

            // Pretty print the properties as JSON
            if let Ok(json_str) = serde_json::to_string_pretty(&delete_event.properties) {
                println!("  📄 Properties:");
                for line in json_str.lines() {
                    println!("    {}", line);
                }
            }
        }
        Event::SessionPeriodError(error_event) => {
            println!("  ❌ Event Type: SessionError");
            println!("  📝 Type: {}", error_event.r#type);

            // Pretty print the properties as JSON
            if let Ok(json_str) = serde_json::to_string_pretty(&error_event.properties) {
                println!("  📄 Error Properties:");
                for line in json_str.lines() {
                    println!("    {}", line);
                }
            }
        }
        Event::SessionPeriodIdle(idle_event) => {
            println!("  😴 Event Type: SessionIdle");
            println!("  📝 Type: {}", idle_event.r#type);

            // Pretty print the properties as JSON
            if let Ok(json_str) = serde_json::to_string_pretty(&idle_event.properties) {
                println!("  📄 Properties:");
                for line in json_str.lines() {
                    println!("    {}", line);
                }
            }
        }
        Event::FilePeriodEdited(file_event) => {
            println!("  📝 Event Type: FileEdited");
            println!("  📝 Type: {}", file_event.r#type);

            // Pretty print the properties as JSON
            if let Ok(json_str) = serde_json::to_string_pretty(&file_event.properties) {
                println!("  📄 File Properties:");
                for line in json_str.lines() {
                    println!("    {}", line);
                }
            }
        }
        Event::FilePeriodWatcherPeriodUpdated(watcher_event) => {
            println!("  👁️  Event Type: FileWatcherUpdated");
            println!("  📝 Type: {}", watcher_event.r#type);

            // Pretty print the properties as JSON
            if let Ok(json_str) = serde_json::to_string_pretty(&watcher_event.properties) {
                println!("  📄 Watcher Properties:");
                for line in json_str.lines() {
                    println!("    {}", line);
                }
            }
        }
        Event::StoragePeriodWrite(storage_event) => {
            println!("  💾 Event Type: StorageWrite");
            println!("  📝 Type: {}", storage_event.r#type);

            // Pretty print the properties as JSON
            if let Ok(json_str) = serde_json::to_string_pretty(&storage_event.properties) {
                println!("  📄 Storage Properties:");
                for line in json_str.lines() {
                    println!("    {}", line);
                }
            }
        }
        Event::LspPeriodClientPeriodDiagnostics(diag_event) => {
            println!("  🔍 Event Type: LspClientDiagnostics");
            println!("  📝 Type: {}", diag_event.r#type);

            // Pretty print the properties as JSON
            if let Ok(json_str) = serde_json::to_string_pretty(&diag_event.properties) {
                println!("  📄 Diagnostics Properties:");
                for line in json_str.lines() {
                    println!("    {}", line);
                }
            }
        }
        Event::InstallationPeriodUpdated(install_event) => {
            println!("  📦 Event Type: InstallationUpdated");
            println!("  📝 Type: {}", install_event.r#type);

            // Pretty print the properties as JSON
            if let Ok(json_str) = serde_json::to_string_pretty(&install_event.properties) {
                println!("  📄 Installation Properties:");
                for line in json_str.lines() {
                    println!("    {}", line);
                }
            }
        }
        Event::IdePeriodInstalled(ide_event) => {
            println!("  🖥️  Event Type: IdeInstalled");
            println!("  📝 Type: {}", ide_event.r#type);

            // Pretty print the properties as JSON
            if let Ok(json_str) = serde_json::to_string_pretty(&ide_event.properties) {
                println!("  📄 IDE Properties:");
                for line in json_str.lines() {
                    println!("    {}", line);
                }
            }
        }
        Event::PermissionPeriodUpdated(perm_event) => {
            println!("  🔐 Event Type: PermissionUpdated");
            println!("  📝 Type: {}", perm_event.r#type);

            // Pretty print the properties as JSON
            if let Ok(json_str) = serde_json::to_string_pretty(&perm_event.properties) {
                println!("  📄 Permission Properties:");
                for line in json_str.lines() {
                    println!("    {}", line);
                }
            }
        }
    }

    // Also provide raw JSON fallback
    if let Ok(raw_json) = serde_json::to_string_pretty(event) {
        println!("  🔍 Raw JSON:");
        for line in raw_json.lines() {
            println!("    {}", line);
        }
    }
}
