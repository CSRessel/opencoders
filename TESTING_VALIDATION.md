# Manual Validation Plan for OpenCode Test Suite

## Overview

This guide provides a concrete plan to manually confirm that the OpenCode test suite is properly integrated with the real `opencode` server binary, making valid HTTP API calls across processes, and would fail if there were breaking changes in the server API.

## 1. Verify Real Process Communication

### Step 1.1: Confirm `opencode` Binary Usage
```bash
# Check that tests use the actual opencode binary
grep -r "opencode serve" tests/
# Should show TestServer spawning real opencode processes

# Verify binary exists and is functional
which opencode
opencode --version
```

### Step 1.2: Monitor Process Creation
```bash
# Run tests while monitoring processes
cargo test smoke_test_file_status &
ps aux | grep "opencode serve" | grep -v grep
# Should show actual opencode server processes running on random ports
```

## 2. Validate HTTP API Calls

### Step 2.1: Network Traffic Verification
```bash
# Monitor network activity during tests
sudo netstat -tlnp | grep opencode
# OR
ss -tlnp | grep opencode
# Should show opencode servers listening on test ports (32000-65000 range)
```

### Step 2.2: HTTP Request Inspection
```bash
# Run single test with network capture
sudo tcpdump -i lo port 8080 &
cargo test smoke_test_config_endpoints -- --nocapture
# Should show HTTP GET/POST requests to /config/providers, /app, etc.
```

## 3. Test Failure Sensitivity

### Step 3.1: Break API Contract
```bash
# Temporarily modify opencode server to return different responses
# Test 1: Change file endpoint to return 404 instead of empty content
curl "http://localhost:8080/file?path=nonexistent" 
# Expected: {"content":""}
# If changed to 404: tests should fail

# Test 2: Remove required fields from config/providers response
curl "http://localhost:8080/config/providers" | jq '.providers[0]'
# If 'name' field removed: tests should fail with deserialization error
```

### Step 3.2: Port Conflict Test
```bash
# Start a dummy server on a port, then run tests
python3 -m http.server 8080 &
cargo test smoke_test_basic_connectivity_health
# Should fail or use different port (tests use random ports 32000+)
```

## 4. Cross-Process Validation

### Step 4.1: Manual API Verification
```bash
# Start server manually and verify same responses tests expect
opencode serve --port 9999 --hostname 127.0.0.1 &

# Test each endpoint the tests use:
curl "http://127.0.0.1:9999/app"                    # Should return app info
curl "http://127.0.0.1:9999/config"                 # Should return config
curl "http://127.0.0.1:9999/config/providers"       # Should return providers
curl "http://127.0.0.1:9999/file/status"           # Should return file list
curl "http://127.0.0.1:9999/file?path=nonexistent" # Should return {"content":""}

pkill -f "opencode serve"
```

### Step 4.2: Version Compatibility Check
```bash
# Verify tests work with current opencode version
opencode --version
cargo test --verbose 2>&1 | grep "Starting test server"
# Should show successful server starts with current opencode binary
```

## 5. Integration Validation Checklist

### ✅ Confirm these behaviors:
- [ ] Tests spawn real `opencode serve` processes (not mocks)
- [ ] Each test uses unique random ports (32000-65000 range)
- [ ] HTTP requests go over actual network stack (localhost)
- [ ] Server processes are properly cleaned up after tests
- [ ] Tests fail when API responses change format
- [ ] Tests fail when required endpoints return errors
- [ ] Multiple tests can run concurrently without port conflicts

### ✅ Red flags that indicate fake/mock testing:
- [ ] No `opencode serve` processes visible during test runs
- [ ] No network ports opened during tests
- [ ] Tests pass when `opencode` binary is missing/broken
- [ ] Tests pass when API responses are malformed
- [ ] All tests use same hardcoded port

## 6. Final Validation Command

```bash
# Single command to verify everything works end-to-end
timeout 30s bash -c '
  cargo test smoke_test_config_endpoints &
  sleep 2
  ps aux | grep "opencode serve" | grep -v grep
  netstat -tlnp | grep opencode
  wait
'
```

This should show:
1. Real opencode processes running
2. Network ports being used
3. Test completing successfully

If any step fails, the tests are not properly integrated with the real opencode server.