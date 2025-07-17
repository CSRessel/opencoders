#!/bin/bash

set -e

echo "🔧 Generating Rust SDK from OpenAPI spec..."

# Check if openapi.json exists
if [ ! -f "openapi.json" ]; then
    echo "❌ openapi.json not found. Run 'make generate-openapi' first."
    exit 1
fi

# Check if generated SDK already exists and is newer than openapi.json
if [ -d "src/sdk/generated" ] && [ "src/sdk/generated" -nt "openapi.json" ]; then
    echo "✅ SDK is up to date (generated files newer than openapi.json)"
    exit 0
fi

# Ensure openapi-generator-cli is installed
if ! command -v openapi-generator-cli &> /dev/null; then
    echo "📦 Installing openapi-generator-cli..."
    if command -v npm &> /dev/null; then
        npm install -g @openapitools/openapi-generator-cli
    else
        echo "❌ npm not found. Please install Node.js and npm first."
        exit 1
    fi
fi

# Create src/sdk directory if it doesn't exist
mkdir -p src/sdk

# Clean previous generation
echo "🧹 Cleaning previous generated code..."
rm -rf src/sdk/generated

# Generate the SDK
echo "⚙️  Generating SDK with openapi-generator-cli..."
openapi-generator-cli generate \
    -i openapi.json \
    -g rust \
    -o src/sdk/generated \
    --additional-properties=packageName=opencode-sdk,packageVersion=0.1.0 \
    --additional-properties=library=reqwest \
    --additional-properties=supportAsync=true \
    --additional-properties=preferUnsignedInt=true \
    --additional-properties=useSingleRequestParameter=true \
    --skip-validate-spec \
    --global-property=apiTests=false,modelTests=false,apiDocs=false,modelDocs=false

# Post-process generated code
echo "🔧 Post-processing generated code..."

# Create a proper mod.rs for the generated module
cat > src/sdk/generated/mod.rs << 'EOF'
//! Generated OpenAPI client for opencode
//! 
//! This module contains auto-generated code from the OpenAPI specification.
//! Do not edit these files directly - they will be overwritten on regeneration.

pub mod apis;
pub mod models;

pub use apis::*;
pub use models::*;

// Re-export configuration for convenience
pub use apis::configuration::Configuration;
EOF

# Fix any common issues in generated code
find src/sdk/generated -name "*.rs" -type f -exec sed -i 's/extern crate /use /g' {} \; 2>/dev/null || true

# Make sure the generated code compiles by adding necessary imports
if [ -f "src/sdk/generated/apis/mod.rs" ]; then
    # Add common imports to the APIs module if needed
    if ! grep -q "pub mod configuration;" src/sdk/generated/apis/mod.rs; then
        sed -i '1i pub mod configuration;' src/sdk/generated/apis/mod.rs
    fi
fi

echo "✅ SDK generation complete!"
echo "📁 Generated files in: src/sdk/generated/"
echo "🔍 Run 'make check' to verify the generated code compiles correctly."