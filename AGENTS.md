# Opencode Rust TUI

## Overview

Rust TUI frontend for `opencode` that communicates with a headless Node.js server. Supports both fullscreen and inline terminal modes with seamless editor integration.

## Critical Functionality

- **Multiple Terminal Modes**: Fullscreen (alternate screen) and inline (within terminal history)
- **API Communication**: Type-safe HTTP client for Node.js backend via auto-generated SDK
- **Event Handling**: Centralized async event processing with TEA architecture
- **Terminal Safety**: Panic hook system with automatic cleanup prevents terminal corruption on panic

The overall TUI architecture uses TEA + Async for maintainability and
performance.

## Modular Elm Architecture

- **Component System**: UI components implement `Component<State, SubMsg, SubCmd>` trait for self-contained state management
- **Sub-Messages**: Complex components use `Msg::ComponentName(submsg)` pattern with component-specific message types
- **Trait-Based Design**: Components implement `Focusable`, `DynamicSize`, and other behavioral traits as needed

## Development Commands

```bash
cargo check # Fast syntax/type checking
cargo build # Build the project
cargo run   # Run the TUI application
```

Currently the test suite is just the SDK testing. Do NOT waste time running the
test suite for validation of TUI changes.

Frequently use the `cargo check` command to validate development progress on any
parts of the project.

## Project Structure

```
opencoders/
├── opencode-sdk/     # Auto-generated API client (DO NOT EDIT)
├── scripts/          # Build and test automation
├── src/
│   ├── app/          # TEA architecture + async runtime
│   └── sdk/          # API client wrapper
└── tests/            # Integration tests of the SDK
```

## Core Libraries

- **ratatui** + **crossterm**: Terminal UI rendering and control
- **tokio**: Async runtime for non-blocking network calls and I/O
- **reqwest** + **serde**: HTTP client with JSON serialization
- **eyre** + **color_eyre**: Error handling

*New libraries must cross a high bar of necessity and quality to be considered.*

## IMPORTANT

**DOs:**
- Always keep `update()` and `view()` functions pure (zero side effects)
- Always store all state in immutable `Model`, updated via `Msg` → `update()` → `Model`
- Always execute side effects as async `Cmd` data structures
- Always use centralized event polling in `event_subscriptions.rs`
- Always communicate with backend via strongly-typed structs from `opencode_sdk::models`
- Always access model state within UI components using `ViewModelContext`
- Always implement complex UI components using the Component trait with sub-messages (e.g., `Msg::TextArea(MsgTextArea::KeyInput)`)
- Explore available API's using the file `openapi.json` and the documentation
`opencode-sdk/README.md`

**DON'Ts:**
- Do NOT call `crossterm::event::read()` outside of `event_subscriptions.rs`
- Do NOT perform I/O or side effects directly in `update()` or `view()` functions
- Do NOT manually edit anything in `opencode-sdk/` directory (auto-generated)
- Do NOT call crossterm terminal functions outside of `terminal.rs`
- Do NOT pass the model into UI components directly
- Do NOT handle component-specific logic directly in the main `update()` function - delegate to component's `update()` method
- Do NOT add dependencies without strong justification and project owner input
