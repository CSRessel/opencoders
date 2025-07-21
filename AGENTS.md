# Opencode Rust TUI

## Overview

This project is a new Terminal User Interface (TUI) frontend for the `opencode` project. The frontend will be a standalone Rust application that communicates with the existing headless javascript server. It replaces the previous Go-based TUI.

The primary goals are to leverage Rust's performance and safety, establish a robust and maintainable architecture, and ensure compatibility with the project's existing backend services.

## Project Structure

The project will be organized into modules to maintain a clean separation of concerns.

```plaintext
opencoders/
├── Cargo.toml               # Project dependencies and metadata
├── Cargo.lock               # Locked dependency versions
├── Makefile                 # Build and development commands
├── README.md                # Project documentation
├── AGENTS.md                # This file - project specification
├── openapi.json             # Static copy of the API spec
├── openapitools.json        # OpenAPI generator configuration
├── opencode-sdk/            # Auto-generated SDK package
│   ├── Cargo.lock
│   ├── README.md    # Generated SDK documentation
│   ├── git_push.sh  # Generated SDK publish script
│   ├── mod.rs       # Generated module root
│   └── src/         # Generated source code
│       ├── lib.rs   # Generated library root
│       ├── apis/    # Generated API client methods
│       │   ├── mod.rs
│       │   ├── configuration.rs
│       │   └── default_api.rs
│       └── models/  # Generated data models (100+ auto-generated structs)
│           ├── mod.rs
│           ├── app.rs
│           ├── config.rs
│           ├── event.rs
│           ├── message.rs
│           ├── session.rs
│           └── ... (90+ other model files)
├── scripts/                 # Build and development scripts
│   ├── generate-openapi.sh  # Script to generate OpenAPI spec
│   ├── generate-sdk.sh      # Script to generate SDK from OpenAPI
│   └── run-smoke-tests.sh   # Script to run integration tests
├── src/                     # Main source code
│   ├── main.rs              # Entry point, calls app::run()
│   ├── lib.rs               # Library root for shared functionality
│   ├── app/                 # TEA architecture with async event handling
│   │   ├── mod.rs           # Public API: run(), INLINE_MODE
│   │   ├── model.rs         # Model struct, AppState, initialization
│   │   ├── msg.rs           # Msg/Cmd/Sub enums for messaging
│   │   ├── update.rs        # Pure update: (Model, Msg) -> (Model, Cmd)
│   │   ├── view.rs          # Pure view: render(Model, Frame)
│   │   ├── subscriptions.rs # Event polling, crossterm → Msg translation
│   │   ├── program.rs       # Async TEA runtime with tokio::select!
│   │   ├── terminal.rs      # Terminal setup/cleanup, TerminalGuard
│   │   └── components/      # Reusable UI components
│   └── sdk/                 # Wrapping functionality around server SDK
│       ├── mod.rs           # SDK module root
│       ├── client.rs        # Main API client implementation
│       ├── error.rs         # SDK error types and handling
│       └── extensions/      # Custom SDK extensions
│           ├── mod.rs
│           └── events.rs    # Event handling extensions
├── target/                  # Rust build artifacts and cache
└── tests/                   # Integration and smoke tests
    ├── README.md            # Test documentation
    ├── common/              # Shared test utilities
    │   ├── mod.rs
    │   ├── assertions.rs    # Common test assertions
    │   └── server.rs        # Test server setup
    ├── file_tests.rs        # File operation tests
    ├── search_tests.rs      # Search functionality tests
    ├── session_tests.rs     # Session management tests
    ├── simple_smoke_test.rs # Basic functionality test
    └── smoke_tests.rs       # Comprehensive integration tests
```

## Technology Stack & Libraries

Library selection is extremely minimal, providing the minimal core pillars to
build on. New libraries must cross a high bar of necessity and quality to be
considered for the project.

| Library / Tool        | Purpose                                                                                                                                  |
| --------------------- | ---------------------------------------------------------------------------------------------------------------------------------------- |
| **Rust** | The core programming language for the project.                                                                                           |
| **`ratatui`** | The low-level TUI library for rendering widgets and managing layouts in the terminal.                                                    |
| **`crossterm`** | The terminal manipulation backend for `ratatui`. It handles raw mode, input events, and platform-specific terminal control.               |
| **`tokio`** | The asynchronous runtime for managing concurrent operations, primarily for handling user input and network I/O without blocking the UI. |
| **`reqwest`** | A high-level, ergonomic HTTP client for communicating with the Node.js backend API.                                                      |
| **`serde`** | A framework for serializing and deserializing Rust data structures to and from JSON for API communication.                               |
| **`anyhow`** | A library for flexible and easy-to-use error handling.                                                                                   |
| **`openapi.json`** | A static copy of the server's OpenAPI specification, used as a reference for creating type-safe API client functions and data models.  |

## Core Architecture

The application follows **The Elm Architecture (TEA)** with **Async Event Handling** and **Centralized Message Passing**. This combines TEA's predictability with async concurrency and ratatui's event handling best practices.

### TEA Components

- **Model**: Immutable state container (`src/app/model.rs`)
- **Messages**: Domain events (`src/app/msg.rs`) - `Msg`, `Cmd`, `Sub` enums
- **Update**: Pure function `(Model, Msg) -> (Model, Cmd)` (`src/app/update.rs`)
- **View**: Pure rendering function (`src/app/view.rs`)

### Event Architecture

- **Centralized Catching**: Single event polling in `subscriptions.rs`
- **Message Translation**: `crossterm::Event` → `Msg` conversion
- **Async Runtime**: Non-blocking I/O, concurrent command execution
- **Command System**: Side effects as data, executed asynchronously

### `src/app/` Module Structure

```text
src/app/
├── mod.rs           // Public API: run(), INLINE_MODE constant
├── model.rs         // Model struct, AppState enum, initialization
├── msg.rs           // Msg/Cmd/Sub enums for TEA messaging
├── update.rs        // Pure update function: (Model, Msg) -> (Model, Cmd)
├── view.rs          // Pure view function: render(Model, Frame)
├── subscriptions.rs // Event polling, crossterm → Msg translation
├── program.rs       // Async TEA runtime, tokio::select! event loop
├── terminal.rs      // Terminal setup/cleanup, TerminalGuard RAII
└── components/      // Reusable UI components
    ├── mod.rs
    └── text_input.rs
```

### Rules

1. **Pure Functions**: `update()` and `view()` have zero side effects
2. **Single State**: All state lives in `Model`, updated immutably
3. **Message-Driven**: All changes flow through `Msg` → `update()` → `Model`
4. **Async Commands**: Side effects execute concurrently via `Cmd` system
5. **Centralized Events**: Only `subscriptions.rs` calls `crossterm::event::read()`

## Key Implementation Details

### API Communication

- HTTP requests via `ApiClient` in `src/sdk/`
- Async `reqwest` operations executed as `Cmd::ApiCall`
- Strongly-typed `serde` structs from `openapi.json`
- API responses become `Msg::ApiResponse` messages

### Terminal Handling

- **Alternate Screen**: Full terminal takeover (default)
- **Inline Mode**: Render within terminal history
- Temporarily set by `INLINE_MODE` constant in `src/app/mod.rs`
- `TerminalGuard` RAII pattern ensures cleanup on panic
- Only `terminal.rs` directly calls crossterm terminal functions
