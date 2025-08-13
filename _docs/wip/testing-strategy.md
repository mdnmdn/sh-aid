# Shaid Testing Strategy

## Testing Philosophy

The testing strategy for Shaid follows a comprehensive approach ensuring reliability, security, and cross-platform compatibility. We aim for high test coverage while maintaining practical and maintainable test suites.

## Testing Pyramid

```
                  ┌─────────────┐
                  │     E2E     │  ← Manual/Automated full system tests
                  └─────────────┘
                ┌─────────────────┐
                │  Integration    │  ← Component interaction tests
                └─────────────────┘
            ┌───────────────────────┐
            │     Unit Tests        │  ← Individual function/module tests
            └───────────────────────┘
```

## Test Categories

### 1. Unit Tests (70% of test effort)

**Scope:** Individual functions, methods, and modules tested in isolation.

#### Configuration Module Tests (`tests/unit/config_tests.rs`)
```rust
#[cfg(test)]
mod config_tests {
    use super::*;
    
    #[test]
    fn test_load_default_config() { /* ... */ }
    
    #[test]
    fn test_config_with_env_vars() { /* ... */ }
    
    #[test]
    fn test_invalid_json_handling() { /* ... */ }
    
    #[test]
    fn test_missing_config_file() { /* ... */ }
}
```

**Test Cases:**
- Default configuration creation
- JSON parsing and validation
- Environment variable fallbacks
- Invalid configuration handling
- File system permission errors
- Cross-platform path handling

#### System Context Module Tests (`tests/unit/context_tests.rs`)
**Test Cases:**
- System information gathering
- Directory listing command execution
- Error handling for failed commands
- Context string formatting
- Performance under different system loads

#### Provider Tests (`tests/unit/provider_tests.rs`)
**Test Cases:**
- Rig agent creation and configuration
- Prompt formatting and execution
- Error handling with Rig's error types
- Provider switching and selection logic
- Configuration validation for different providers

### 2. Integration Tests (25% of test effort)

**Scope:** Testing interactions between modules and external dependencies.

#### Full Workflow Tests (`tests/integration/workflow_tests.rs`)
```rust
#[tokio::test]
async fn test_complete_command_generation() {
    // Test entire flow: config → context → provider → output
}

#[tokio::test]
async fn test_provider_switching() {
    // Test fallback between providers
}
```

**Test Cases:**
- Complete command generation workflow
- Provider fallback mechanisms
- Configuration changes during runtime
- Error propagation through system layers
- Resource cleanup after failures

#### Rig Integration Tests (`tests/integration/api_tests.rs`)
**Test Cases:**
- Real Rig agent calls with test credentials
- Rig's built-in retry and timeout handling
- Provider-specific model availability
- Configuration edge cases with Rig providers
- Performance comparison between providers

**Note:** Integration tests will use dedicated test API keys and leverage Rig's built-in rate limiting and error handling.

### 3. End-to-End Tests (5% of test effort)

**Scope:** Full system testing from CLI input to command output.

#### CLI Tests (`tests/e2e/cli_tests.rs`)
```rust
#[test]
fn test_cli_with_valid_input() {
    let output = Command::new("./target/debug/shaid")
        .args(&["list files in current directory"])
        .output()
        .expect("Failed to execute command");
    
    assert!(output.status.success());
    // Validate output format
}
```

**Test Cases:**
- Command-line argument parsing
- Invalid input handling
- Configuration file discovery
- Output formatting
- Exit codes and error messages

## Mock Strategy

### Mock Implementations

#### Mock HTTP Client
```rust
pub struct MockHttpClient {
    responses: HashMap<String, Result<String, HttpError>>,
}

impl MockHttpClient {
    pub fn with_response(url: &str, response: String) -> Self { /* ... */ }
    pub fn with_error(url: &str, error: HttpError) -> Self { /* ... */ }
}
```

#### Mock Rig Agents
```rust
use rig::completion::Prompt;

pub struct MockRigAgent {
    responses: std::sync::Mutex<VecDeque<Result<String, rig::completion::CompletionError>>>,
}

#[async_trait]
impl rig::Agent for MockRigAgent {
    async fn prompt(&self, _prompt: &str) -> Result<String, rig::completion::CompletionError> {
        self.responses.lock().unwrap()
            .pop_front()
            .unwrap_or_else(|| Ok("ls -la".to_string()))
    }
    
    async fn chat(&self, _prompt: &str, _chat_history: rig::completion::ChatHistory) 
        -> Result<String, rig::completion::CompletionError> {
        self.prompt(_prompt).await
    }
}
```

### Mock Usage Patterns
1. **Deterministic Testing**: Predictable responses for consistent test results
2. **Error Simulation**: Test error handling without depending on external failures
3. **Performance Testing**: Eliminate network latency for performance tests
4. **Offline Development**: Enable development and testing without internet access

## Cross-Platform Testing

### Target Platforms
- **Linux**: Ubuntu 20.04+, CentOS 8+, Debian 11+
- **macOS**: macOS 11+ (Intel and Apple Silicon)
- **Windows**: Windows 10+

### Platform-Specific Tests

#### File System Tests
```rust
#[cfg(target_os = "windows")]
#[test]
fn test_windows_config_path() {
    // Test Windows-specific config directory handling
}

#[cfg(target_family = "unix")]
#[test]
fn test_unix_permissions() {
    // Test Unix file permissions handling
}
```

#### System Command Tests
- Shell detection (bash, zsh, cmd, PowerShell)
- Path separator handling
- Command execution differences
- Environment variable access

### CI/CD Platform Matrix
```yaml
strategy:
  matrix:
    os: [ubuntu-latest, macos-latest, windows-latest]
    rust: [stable, beta]
```

## Security Testing

### Security Test Categories

#### Input Validation Tests
```rust
#[test]
fn test_command_injection_prevention() {
    let malicious_input = "list files; rm -rf /";
    let result = generate_command(&config, malicious_input, &context).await;
    // Ensure no dangerous commands are generated
}
```

#### API Key Security Tests
```rust
#[test]
fn test_api_key_not_in_logs() {
    // Ensure API keys never appear in logs or error messages
}

#[test]
fn test_config_file_permissions() {
    // Verify config files have appropriate permissions
}
```

#### Output Sanitization Tests
- Ensure no sensitive information in command output
- Validate command safety checks
- Test path traversal prevention

## Performance Testing

### Performance Benchmarks

#### Startup Performance
```rust
#[bench]
fn bench_config_loading(b: &mut Bencher) {
    b.iter(|| {
        load_config()
    });
}

#[bench]
fn bench_context_gathering(b: &mut Bencher) {
    b.iter(|| {
        gather_system_context()
    });
}
```

#### Response Time Benchmarks
- Command generation latency
- Configuration loading time
- System context gathering performance
- Memory usage profiling

### Performance Targets
- **Cold start**: <500ms from CLI invocation to API call
- **Config loading**: <50ms for typical config files
- **Context gathering**: <100ms for system information collection
- **Memory usage**: <50MB peak memory usage

## Test Data Management

### Test Fixtures
```
tests/fixtures/
├── configs/
│   ├── valid_config.json
│   ├── invalid_config.json
│   └── empty_config.json
├── responses/
│   ├── openai_success.json
│   ├── claude_error.json
│   └── gemini_response.json
└── system/
    ├── linux_context.txt
    ├── macos_context.txt
    └── windows_context.txt
```

### Test Configuration
- Use dedicated test API keys (rate-limited)
- Environment-specific test configurations
- Automated test data cleanup
- Version-controlled test fixtures

## Continuous Integration

### CI Pipeline Stages

1. **Static Analysis**
   - `cargo fmt --check`
   - `cargo clippy -- -D warnings`
   - Security audit with `cargo audit`

2. **Unit Tests**
   - `cargo test --lib`
   - Coverage reporting with `cargo tarpaulin`

3. **Integration Tests**
   - `cargo test --test '*'`
   - Real API integration tests (limited)

4. **Cross-Platform Builds**
   - Build verification on all target platforms
   - Binary size and performance regression checks

5. **Security Scans**
   - Dependency vulnerability scanning
   - Static security analysis

### Test Reporting
- Coverage reports with codecov integration
- Performance regression alerts
- Cross-platform compatibility reports
- Security scan results

## Test Maintenance

### Review Process
- All tests reviewed in pull requests
- Test coverage requirements (minimum 80%)
- Performance regression prevention
- Regular test suite cleanup

### Test Documentation
- Test purpose and scope documentation
- Mock setup and usage guidelines
- Platform-specific testing notes
- Troubleshooting common test failures

This comprehensive testing strategy ensures Shaid's reliability, security, and performance across all supported platforms while maintaining development velocity.