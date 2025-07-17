# OpenCoders

A Rust-based client for the opencode API.

## OpenAPI Specification Generation

This package includes scripts to automatically generate the OpenAPI specification from the opencode server. The generated `openapi.json` file can be used for client code generation and API documentation.

### Usage

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

#### Using Scripts Directly

**Bash Script:**

```bash
./scripts/generate-openapi.sh
```

#### CI Integration

For continuous integration, you can use either script:

```yaml
# GitHub Actions example
- name: Generate OpenAPI spec
  run: |
    cd packages/opencoders
    make generate-openapi

- name: Build opencoders
  run: |
    cd packages/opencoders
    make build
```

## Development

The generated `openapi.json` file is excluded from version control via `.gitignore`. Always run the generation scripts before building or testing to ensure you have the latest API specification.
