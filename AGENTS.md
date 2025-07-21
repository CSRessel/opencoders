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
│   ├── main.rs              # Entry point, terminal setup, main event loop
│   ├── lib.rs               # Library root for shared functionality
│   ├── app/                 # Business logic and TUI architecture (to be implemented)
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

The application will strictly follow **The Elm Architecture**, a functional design pattern that separates state, logic, and rendering. This promotes predictability and simplifies state management.

The architecture consists of three main parts:

  - **Model**: A single Rust `struct` (e.g., `App`) that holds the entire state of the application. This includes user input, message history, file system state, agent status, etc. It is the single source of truth.

  - **Update**: A function or method (e.g., `App::update`) that contains all business logic. It takes the current `Model` and an `Event` as input and produces a new `Model`. It is the only part of the application that can modify the state.

  - **View**: A function (e.g., `ui`) that renders the user interface based on the current `Model`. It is stateless and declarative, receiving the `Model` and drawing to the terminal using `ratatui`. It does not contain any application logic.

An asynchronous main event loop will drive the application, polling for user input and network events, dispatching them to the **Update** function, and triggering a re-render by the **View** function.

As the application grows, the `Model`, `Update`, and `View` components may be extracted from `main.rs` into their own dedicated modules (e.g., `app.rs`, `ui.rs`, `event.rs`, and similar), structured within the `src/app/` folder.

## Key Implementation Details

### API Communication

  - All communication with the backend will be via HTTP requests made by the `ApiClient` defined in `src/sdk/`.
  - The `ApiClient` will use `reqwest` to perform asynchronous `POST`, `GET`, etc. operations.
  - All JSON request payloads and response bodies will be represented by strongly-typed Rust structs using `serde::Serialize` and `serde::Deserialize`.
  - The API contract is defined by the `openapi.json` file, which is generated by calling into the server's command line functionality.

### Terminal Handling (Alternate vs. Inline Mode)

  - The TUI must support running in two modes:
    1.  **Alternate Screen**: The default mode, where the TUI takes over the
        full terminal window, and can place dynamic UI features at all parts of
        the window.
    2.  **Inline Mode**: The TUI renders within the existing terminal history,
        and rewrites lines as necessary in the bottom most area of the terminal
        for dyanmic UI features..
  - This will be controlled via a command-line flag. The main function will conditionally execute `crossterm`'s `EnterAlternateScreen` and `LeaveAlternateScreen` commands based on this flag.
  - Terminal setup and restoration will be handled in a way that guarantees restoration even in the event of an application panic.
