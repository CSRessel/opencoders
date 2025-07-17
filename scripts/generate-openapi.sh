#!/bin/bash

set -e

echo "Generating OpenAPI spec using opencode CLI..."

# Get the directory of this script
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Navigate to project root (3 levels up from scripts/)
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"

# Change to project root directory
cd "$PROJECT_ROOT"

# Run the existing generate command
echo "Running: bun run ./packages/opencode/src/index.ts generate"
bun run ./packages/opencode/src/index.ts generate

# Copy the generated spec to opencoders package
echo "Copying generated spec to opencoders package..."
cp gen/openapi.json packages/opencoders/openapi.json

echo "✅ OpenAPI spec generated at packages/opencoders/openapi.json"