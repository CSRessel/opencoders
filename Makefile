.PHONY: generate-openapi generate-sdk build clean help

# Generate OpenAPI specification (only if needed)
generate-openapi:
	@echo "Checking OpenAPI specification..."
	@./scripts/generate-openapi.sh

# Force regenerate OpenAPI specification
generate-openapi-force:
	@echo "Force generating OpenAPI specification..."
	@./scripts/generate-openapi.sh --force

# Generate SDK from OpenAPI spec
generate-sdk:
	@echo "Generating Rust SDK from OpenAPI specification..."
	@chmod +x scripts/generate-sdk.sh
	@./scripts/generate-sdk.sh

# Force regenerate SDK (cleans first)
generate-sdk-force:
	@echo "Force regenerating Rust SDK..."
	@rm -rf opencode-sdk
	@chmod +x scripts/generate-sdk.sh
	@./scripts/generate-sdk.sh

# Build the Rust project (generates SDK if needed)
build: generate-sdk
	@echo "Building opencoders..."
	@cargo build

# Build release version (generates SDK if needed)
build-release: generate-sdk
	@echo "Building opencoders (release)..."
	@cargo build --release

# Build with fresh OpenAPI and SDK generation
build-with-openapi: generate-openapi generate-sdk-force build

# Build release with fresh OpenAPI and SDK generation
build-release-with-openapi: generate-openapi generate-sdk-force build-release

# Clean build artifacts (preserves committed files)
clean:
	@echo "Cleaning build artifacts..."
	@cargo clean

# Clean everything including generated files
clean-all: clean
	@echo "Removing generated files..."
	@rm -f openapi.json
	@rm -rf opencode-sdk

# Run tests (generates SDK if needed)
test: generate-sdk
	@echo "Running tests..."
	@cargo test

# Run smoke tests (generates SDK if needed)
test-smoke: generate-sdk
	@echo "Running smoke tests..."
	@cargo test --test simple_smoke_test -- --test-threads=1

# Run smoke tests with verbose output
test-smoke-verbose: generate-sdk
	@echo "Running smoke tests with verbose output..."
	@cargo test --test simple_smoke_test -- --test-threads=1 --nocapture

# Run specific smoke test category
test-smoke-basic: generate-sdk
	@echo "Running basic connectivity smoke tests..."
	@cargo test --test smoke_tests -- --nocapture

test-smoke-sessions: generate-sdk
	@echo "Running session management smoke tests..."
	@cargo test --test session_tests -- --nocapture

test-smoke-files: generate-sdk
	@echo "Running file operations smoke tests..."
	@cargo test --test file_tests -- --nocapture

test-smoke-search: generate-sdk
	@echo "Running search operations smoke tests..."
	@cargo test --test search_tests -- --nocapture

# Run tests with fresh OpenAPI and SDK generation
test-with-openapi: generate-openapi generate-sdk-force test

# Run smoke tests with fresh OpenAPI and SDK generation
test-smoke-with-openapi: generate-openapi generate-sdk-force test-smoke

# Development workflow - quick iteration
dev: generate-sdk
	@echo "Running development build..."
	@cargo run

# Check generated code
check: generate-sdk
	@echo "Checking generated code..."
	@cargo check
	@cargo clippy -- -D warnings

# Show available commands
help:
	@echo "Available commands:"
	@echo "  generate-openapi         Generate OpenAPI specification (only if needed)"
	@echo "  generate-openapi-force   Force regenerate OpenAPI specification"
	@echo "  generate-sdk             Generate Rust SDK from OpenAPI spec"
	@echo "  generate-sdk-force       Force regenerate Rust SDK (cleans first)"
	@echo "  build                    Build the project (generates SDK if needed)"
	@echo "  build-release            Build release version (generates SDK if needed)"
	@echo "  build-with-openapi       Build with fresh OpenAPI and SDK generation"
	@echo "  build-release-with-openapi Build release with fresh OpenAPI and SDK generation"
	@echo "  test                     Run tests (generates SDK if needed)"
	@echo "  test-smoke               Run smoke tests against real opencode server"
	@echo "  test-smoke-verbose       Run smoke tests with verbose output"
	@echo "  test-smoke-basic         Run basic connectivity smoke tests"
	@echo "  test-smoke-sessions      Run session management smoke tests"
	@echo "  test-smoke-files         Run file operations smoke tests"
	@echo "  test-smoke-search        Run search operations smoke tests"
	@echo "  test-with-openapi        Run tests with fresh OpenAPI and SDK generation"
	@echo "  test-smoke-with-openapi  Run smoke tests with fresh OpenAPI and SDK generation"
	@echo "  dev                      Development workflow - quick iteration"
	@echo "  check                    Check generated code with clippy"
	@echo "  clean                    Clean build artifacts (preserves committed files)"
	@echo "  clean-all                Clean everything including generated files"
	@echo "  help                     Show this help message"

# Default target
all: build
