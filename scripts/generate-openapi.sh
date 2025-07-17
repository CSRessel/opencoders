#!/bin/bash

set -e

echo "Generating OpenAPI spec using opencode CLI..."

# Get the directory of this script
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Navigate to project root (3 levels up from scripts/)
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"

# Change to project root directory
cd "$PROJECT_ROOT"

# Check if we need to regenerate (if openapi.json doesn't exist or if source files are newer)
OPENAPI_FILE="packages/opencoders/openapi.json"
NEEDS_REGEN=false

if [ ! -f "$OPENAPI_FILE" ]; then
    echo "OpenAPI spec doesn't exist, generating..."
    NEEDS_REGEN=true
elif [ -n "$(find packages/opencode/src -name '*.ts' -newer "$OPENAPI_FILE" 2>/dev/null)" ]; then
    echo "Source files are newer than OpenAPI spec, regenerating..."
    NEEDS_REGEN=true
else
    echo "OpenAPI spec is up-to-date, skipping generation"
fi

if [ "$NEEDS_REGEN" = true ] || [ "$1" = "--force" ]; then
    # Run the existing generate command
    echo "Running: bun run ./packages/opencode/src/index.ts generate"
    bun run ./packages/opencode/src/index.ts generate

    # Copy the generated spec to opencoders package
    echo "Copying generated spec to opencoders package..."
    cp gen/openapi.json packages/opencoders/openapi.json

    echo "✅ OpenAPI spec generated at packages/opencoders/openapi.json"
else
    echo "✅ Using existing OpenAPI spec at packages/opencoders/openapi.json"
fi