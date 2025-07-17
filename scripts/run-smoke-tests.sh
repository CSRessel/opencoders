#!/bin/bash

# OpenCode SDK Smoke Test Runner
# This script runs the smoke tests for the OpenCode SDK against a real server

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
TIMEOUT=${SMOKE_TEST_TIMEOUT:-300}  # 5 minutes default timeout
VERBOSE=${SMOKE_TEST_VERBOSE:-false}
PARALLEL=${SMOKE_TEST_PARALLEL:-false}

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to check prerequisites
check_prerequisites() {
    print_status "Checking prerequisites..."
    
    # Check if opencode is available
    if ! command -v opencode &> /dev/null; then
        print_error "opencode command not found in PATH"
        print_error "Please install opencode or ensure it's available in your PATH"
        exit 1
    fi
    
    # Check opencode version
    local version=$(opencode --version 2>/dev/null || echo "unknown")
    print_status "Found opencode version: $version"
    
    # Check if cargo is available
    if ! command -v cargo &> /dev/null; then
        print_error "cargo command not found in PATH"
        print_error "Please install Rust and Cargo"
        exit 1
    fi
    
    print_success "Prerequisites check passed"
}

# Function to build the project
build_project() {
    print_status "Building project and generating SDK..."
    
    if ! make generate-sdk; then
        print_error "Failed to generate SDK"
        exit 1
    fi
    
    if ! cargo build; then
        print_error "Failed to build project"
        exit 1
    fi
    
    print_success "Project built successfully"
}

# Function to run smoke tests
run_smoke_tests() {
    print_status "Running smoke tests..."
    
    local test_args=""
    if [ "$VERBOSE" = "true" ]; then
        test_args="$test_args --nocapture"
    fi
    
    if [ "$PARALLEL" = "false" ]; then
        test_args="$test_args --test-threads=1"
    fi
    
    local test_files=(
        "simple_smoke_test"
    )
    
    local failed_tests=()
    local passed_tests=()
    
    for test_file in "${test_files[@]}"; do
        print_status "Running $test_file..."
        
        if timeout $TIMEOUT cargo test --test "$test_file" -- $test_args; then
            passed_tests+=("$test_file")
            print_success "$test_file passed"
        else
            failed_tests+=("$test_file")
            print_error "$test_file failed"
        fi
    done
    
    # Print summary
    echo
    print_status "Test Summary:"
    echo "  Passed: ${#passed_tests[@]}"
    echo "  Failed: ${#failed_tests[@]}"
    
    if [ ${#passed_tests[@]} -gt 0 ]; then
        print_success "Passed tests: ${passed_tests[*]}"
    fi
    
    if [ ${#failed_tests[@]} -gt 0 ]; then
        print_error "Failed tests: ${failed_tests[*]}"
        return 1
    fi
    
    return 0
}

# Function to show usage
show_usage() {
    echo "Usage: $0 [OPTIONS]"
    echo
    echo "Options:"
    echo "  -v, --verbose     Enable verbose output"
    echo "  -p, --parallel    Run tests in parallel (default: serial)"
    echo "  -t, --timeout N   Set timeout in seconds (default: 300)"
    echo "  -h, --help        Show this help message"
    echo
    echo "Environment Variables:"
    echo "  SMOKE_TEST_TIMEOUT   Test timeout in seconds (default: 300)"
    echo "  SMOKE_TEST_VERBOSE   Enable verbose output (true/false)"
    echo "  SMOKE_TEST_PARALLEL  Run tests in parallel (true/false)"
    echo
    echo "Examples:"
    echo "  $0                    # Run tests with default settings"
    echo "  $0 --verbose          # Run with verbose output"
    echo "  $0 --parallel         # Run tests in parallel"
    echo "  $0 --timeout 600      # Set 10 minute timeout"
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -v|--verbose)
            VERBOSE=true
            shift
            ;;
        -p|--parallel)
            PARALLEL=true
            shift
            ;;
        -t|--timeout)
            TIMEOUT="$2"
            shift 2
            ;;
        -h|--help)
            show_usage
            exit 0
            ;;
        *)
            print_error "Unknown option: $1"
            show_usage
            exit 1
            ;;
    esac
done

# Main execution
main() {
    print_status "Starting OpenCode SDK smoke tests"
    print_status "Configuration: timeout=${TIMEOUT}s, verbose=${VERBOSE}, parallel=${PARALLEL}"
    
    check_prerequisites
    build_project
    
    if run_smoke_tests; then
        print_success "All smoke tests passed!"
        exit 0
    else
        print_error "Some smoke tests failed!"
        exit 1
    fi
}

# Run main function
main "$@"