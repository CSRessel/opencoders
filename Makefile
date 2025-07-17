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
	@rm -rf src/sdk/generated
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
	@rm -rf src/sdk/generated

# Run tests (generates SDK if needed)
test: generate-sdk
	@echo "Running tests..."
	@cargo test

# Run tests with fresh OpenAPI and SDK generation
test-with-openapi: generate-openapi generate-sdk-force test

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
	@echo "  test-with-openapi        Run tests with fresh OpenAPI and SDK generation"
	@echo "  dev                      Development workflow - quick iteration"
	@echo "  check                    Check generated code with clippy"
	@echo "  clean                    Clean build artifacts (preserves committed files)"
	@echo "  clean-all                Clean everything including generated files"
	@echo "  help                     Show this help message"

# Default target
all: build