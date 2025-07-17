.PHONY: generate-openapi build clean help

# Generate OpenAPI specification
generate-openapi:
	@echo "Generating OpenAPI specification..."
	@./scripts/generate-openapi.sh

# Build the Rust project
build:
	@echo "Building opencoders..."
	@cargo build

# Build release version
build-release:
	@echo "Building opencoders (release)..."
	@cargo build --release

# Build with fresh OpenAPI generation
build-with-openapi: generate-openapi build

# Build release with fresh OpenAPI generation
build-release-with-openapi: generate-openapi build-release

# Clean build artifacts (preserves committed openapi.json)
clean:
	@echo "Cleaning build artifacts..."
	@cargo clean

# Clean everything including generated openapi.json
clean-all: clean
	@echo "Removing generated openapi.json..."
	@rm -f openapi.json

# Run tests
test:
	@echo "Running tests..."
	@cargo test

# Run tests with fresh OpenAPI generation
test-with-openapi: generate-openapi test

# Show available commands
help:
	@echo "Available commands:"
	@echo "  generate-openapi         Generate OpenAPI specification from opencode server"
	@echo "  build                    Build the project (uses committed openapi.json)"
	@echo "  build-release            Build release version (uses committed openapi.json)"
	@echo "  build-with-openapi       Build with fresh OpenAPI generation"
	@echo "  build-release-with-openapi Build release with fresh OpenAPI generation"
	@echo "  test                     Run tests (uses committed openapi.json)"
	@echo "  test-with-openapi        Run tests with fresh OpenAPI generation"
	@echo "  clean                    Clean build artifacts (preserves openapi.json)"
	@echo "  clean-all                Clean everything including openapi.json"
	@echo "  help                     Show this help message"

# Default target
all: build