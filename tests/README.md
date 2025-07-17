# OpenCode SDK Smoke Tests

This directory contains smoke tests for the OpenCode SDK that verify basic connectivity and functionality with a real `opencode` server instance.

## Overview

The smoke tests are designed to:
- Validate that the generated SDK can communicate with the real opencode server
- Test basic API operations without complex mocking
- Ensure error handling works correctly
- Verify concurrent operations work as expected
- Provide fast feedback on SDK functionality

## Test Structure

### Test Files

- `smoke_tests.rs` - Basic connectivity and configuration tests
- `session_tests.rs` - Session lifecycle management tests  
- `file_tests.rs` - File system operation tests
- `search_tests.rs` - Search functionality tests
- `common/` - Shared test utilities and helpers

### Test Utilities

- `TestServer` - Manages opencode server instances for testing
- Custom assertion macros for API result validation
- Port management and server health checking
- Concurrent test execution helpers

## Prerequisites

1. **OpenCode Binary**: The `opencode` command must be available in your PATH
2. **Rust Environment**: Standard Rust development environment with Cargo
3. **Network Access**: Tests use localhost ports for server communication

## Running Tests

### Run All Smoke Tests
```bash
cargo test --test smoke_tests --test session_tests --test file_tests --test search_tests
```

### Run Specific Test Categories
```bash
# Basic connectivity tests
cargo test --test smoke_tests

# Session management tests  
cargo test --test session_tests

# File operation tests
cargo test --test file_tests

# Search functionality tests
cargo test --test search_tests
```

### Run Tests Serially (Recommended for CI)
```bash
cargo test --test smoke_tests --test session_tests --test file_tests --test search_tests -- --test-threads=1
```

### Run with Verbose Output
```bash
cargo test --test smoke_tests -- --nocapture
```

## Test Design Principles

### Real Server Integration
- Tests use actual `opencode` server instances, not mocks
- Each test gets its own server on a unique port
- Server runs in a temporary directory for isolation

### Graceful Error Handling
- Tests expect and handle various error conditions
- Distinguishes between expected failures and actual bugs
- Provides detailed error context for debugging

### Concurrent Safety
- Tests verify that multiple concurrent requests work correctly
- Each test server is isolated to prevent interference
- Port allocation prevents conflicts between parallel tests

### Minimal Dependencies
- Uses only essential test dependencies
- Leverages existing `opencode` binary rather than complex mocking
- Fast execution with minimal setup overhead

## Test Categories

### Basic Connectivity (`smoke_tests.rs`)
- App info retrieval
- Configuration endpoint access
- Provider and mode listing
- Error handling validation
- Concurrent request handling

### Session Management (`session_tests.rs`)
- Session creation and deletion
- Session listing and verification
- Multiple session handling
- Session operation error cases
- Message retrieval for sessions

### File Operations (`file_tests.rs`)
- File status retrieval
- File reading operations
- Error handling for invalid paths
- Concurrent file operations
- File system consistency checks

### Search Operations (`search_tests.rs`)
- File pattern searching
- Text content searching
- Symbol searching (when LSP available)
- Search error handling
- Concurrent search operations

## Troubleshooting

### Common Issues

**Server Start Failures**
- Ensure `opencode` is installed and in PATH
- Check that required ports are available
- Verify sufficient disk space for temporary directories

**Test Timeouts**
- Increase timeout values in `TestConfig`
- Check system performance and load
- Ensure no firewall blocking localhost connections

**Intermittent Failures**
- Run tests serially with `--test-threads=1`
- Check for port conflicts with other services
- Verify stable network connectivity

### Debug Mode

Enable debug output by setting environment variables:
```bash
RUST_LOG=debug cargo test --test smoke_tests -- --nocapture
```

### Manual Server Testing

You can manually start a test server for debugging:
```bash
opencode server --port 8080 --host 127.0.0.1
```

Then test endpoints manually:
```bash
curl http://127.0.0.1:8080/app
curl http://127.0.0.1:8080/config
```

## CI/CD Integration

### GitHub Actions Example
```yaml
- name: Run SDK Smoke Tests
  run: |
    cargo test --test smoke_tests --test session_tests --test file_tests --test search_tests -- --test-threads=1
  env:
    RUST_LOG: info
```

### Test Execution Strategy
- Tests run serially to avoid port conflicts
- Each test category can be run independently
- Failures provide detailed context for debugging
- Tests clean up resources automatically

## Contributing

When adding new smoke tests:

1. Follow the existing test structure and naming conventions
2. Use the `TestServer` utility for server management
3. Include both success and error case testing
4. Add appropriate assertions with descriptive messages
5. Ensure tests clean up resources properly
6. Update this README with new test descriptions

## Performance

Typical test execution times:
- Basic connectivity: ~2-5 seconds per test
- Session management: ~3-8 seconds per test
- File operations: ~2-6 seconds per test
- Search operations: ~3-10 seconds per test

Total suite execution: ~30-60 seconds depending on system performance.