# opencoders

A high-performance, terminal-native client for opencode built with Rust. This TUI provides a responsive interface for interacting with the opencode headless server, leveraging Rust's safety and performance characteristics for a superior developer experience.

## Quick Start

### Prerequisites
- Rust toolchain (1.70+)
- Git
- GitHub CLI (optional, for cloning)
- Bun toolchain (1.2+)

### Installation

```bash
# Clone and setup the main opencode monorepo
gh repo clone sst/opencode
cd opencode/packages/opencode/
bun install

# Navigate back and setup the Rust TUI client
cd ../../
gh repo clone CSRessel/opencoders
cd opencoders
cargo build --release

# Launch the TUI
cargo run
```

<!--
TODO once packaged correctly
(deps on opencode executable on system)

### Alternative Installation
```bash
# Install directly from source
cargo install --git https://github.com/CSRessel/opencoders
opencoders
```
-->

## Features

- **Native Performance**: Built with Rust for minimal resource usage and maximum responsiveness
- **Terminal Integration**: Supports both alternate screen and inline modes for flexible usage
- **Type-Safe API**: Auto-generated client bindings ensure compile-time API compatibility
- **Async Architecture**: Non-blocking I/O keeps the interface responsive during server communication

## API Integration

The client maintains type-safe communication with the opencode server through automatically generated bindings. The OpenAPI specification is dynamically generated from the server to ensure perfect API compatibility.

### OpenAPI Generation

#### Using the Makefile (Recommended)

```bash
# Generate OpenAPI specification
make generate-openapi

# Build the project (includes OpenAPI generation)
make build

# Build release version
make build-release

# Run tests
make test

# Clean build artifacts and generated files
make clean

# Show available commands
make help
```

#### Manual Generation

```bash
# Generate OpenAPI specification directly
./scripts/generate-openapi.sh
```

#### Continuous Integration

Integrate OpenAPI generation into your CI pipeline:

```yaml
# GitHub Actions workflow
name: Build opencoders
on: [push, pull_request]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Generate API bindings
        run: |
          cd packages/opencoders
          make generate-openapi
```

## Development

### Architecture

The application follows **The Elm Architecture** pattern for predictable state management:
- **Model**: Single source of truth for application state
- **Update**: Pure functions handling state transitions
- **View**: Declarative UI rendering with `ratatui`

### Key Dependencies
- `ratatui` - Terminal UI framework
- `tokio` - Async runtime
- `reqwest` - HTTP client for server communication
- `serde` - JSON serialization
- `crossterm` - Cross-platform terminal control

### Building from Source

```bash
# Development build
cargo build

# Optimized release build
cargo build --release

# Run tests
cargo test

# Generate fresh API bindings
make generate-openapi
```
