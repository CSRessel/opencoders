.PHONY: generate-openapi build clean help

# Generate OpenAPI specification
generate-openapi:
	@echo "Generating OpenAPI specification..."
	@./scripts/generate-openapi.sh

# Build the Rust project (depends on OpenAPI generation)
build: generate-openapi
	@echo "Building opencoders..."
	@cargo build

# Build release version
build-release: generate-openapi
	@echo "Building opencoders (release)..."
	@cargo build --release

# Clean build artifacts and generated files
clean:
	@echo "Cleaning build artifacts..."
	@cargo clean
	@rm -f openapi.json

# Run tests
test: generate-openapi
	@echo "Running tests..."
	@cargo test

# Show available commands
help:
	@echo "Available commands:"
	@echo "  generate-openapi  Generate OpenAPI specification from opencode server"
	@echo "  build            Build the project (includes OpenAPI generation)"
	@echo "  build-release    Build release version (includes OpenAPI generation)"
	@echo "  test             Run tests (includes OpenAPI generation)"
	@echo "  clean            Clean build artifacts and generated files"
	@echo "  help             Show this help message"

# Default target
all: build